use super::cost;
use crate::{
    cost::{Basis, Estimate},
    evidence::{AgentObservation, AgentUpdate, Usage},
};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Decoder {
    pub(crate) note: Option<String>,
    finished: bool,
    started_at_ms: u64,
    requests: BTreeMap<(String, String), cost::Request>,
    incomplete: bool,
}
impl Decoder {
    pub fn new(started_at_ms: u64) -> Self {
        Self {
            started_at_ms,
            ..Self::default()
        }
    }
    fn message_usage(&mut self, value: &Value) -> Vec<AgentUpdate> {
        if self.finished {
            return Vec::new();
        }
        let message = &value["message"];
        if message["usage"].is_null() {
            return Vec::new();
        }
        let Some(id) = message["id"].as_str() else {
            return self.incomplete_usage();
        };
        let Some(usage) = usage(&message["usage"]) else {
            return self.incomplete_usage();
        };
        let Some(model) = message["model"].as_str() else {
            return self.incomplete_usage();
        };
        let timestamp_ms = value["timestamp"]
            .as_str()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
            .and_then(|t| u64::try_from(t.timestamp_millis()).ok())
            .unwrap_or(self.started_at_ms);
        // CodexBar replaces repeated message records with their latest cumulative usage.
        self.requests.insert(
            (
                id.into(),
                value["requestId"].as_str().unwrap_or_default().into(),
            ),
            cost::Request {
                model: model.into(),
                usage,
                cache_write_1h: message["usage"]["cache_creation"]["ephemeral_1h_input_tokens"]
                    .as_u64()
                    .unwrap_or(0),
                timestamp_ms,
            },
        );
        let (usage, cost) = self.observed();
        vec![AgentUpdate::UsageSnapshot(usage), AgentUpdate::Cost(cost)]
    }
    fn incomplete_usage(&mut self) -> Vec<AgentUpdate> {
        self.incomplete = true;
        let (_, cost) = self.observed();
        vec![AgentUpdate::UsageSnapshot(None), AgentUpdate::Cost(cost)]
    }
    fn observed(&self) -> (Option<Usage>, Estimate) {
        let mut total = Usage {
            cache_write_input_tokens: Some(0),
            ..Usage::default()
        };
        let mut valid = !self.incomplete && !self.requests.is_empty();
        for row in self.requests.values() {
            for (sum, value) in [
                (&mut total.input_tokens, row.usage.input_tokens),
                (
                    &mut total.cached_input_tokens,
                    row.usage.cached_input_tokens,
                ),
                (&mut total.output_tokens, row.usage.output_tokens),
            ] {
                if let Some(next) = sum.checked_add(value) {
                    *sum = next;
                } else {
                    valid = false;
                }
            }
            total.cache_write_input_tokens = total
                .cache_write_input_tokens
                .zip(row.usage.cache_write_input_tokens)
                .and_then(|(a, b)| a.checked_add(b));
            valid &= total.cache_write_input_tokens.is_some();
        }
        let mut cost = cost::estimate(self.requests.values());
        if !valid {
            cost.usd = None;
            cost.note = "API cost unavailable: observed Claude usage is incomplete.".into();
        }
        (valid.then_some(total), cost)
    }

    pub fn decode(&mut self, line: &[u8]) -> AgentObservation {
        let Ok(value) = serde_json::from_slice::<Value>(line) else {
            return AgentObservation::Invalid;
        };
        let kind = value["type"].as_str().unwrap_or("unknown");
        let update = match kind {
            "system" if value["subtype"] == "init" => {
                AgentUpdate::Thread(value["session_id"].as_str().map(str::to_owned))
            }
            "assistant" => {
                let mut updates = self.message_usage(&value);
                let content = value["message"]["content"].as_array();
                let text = content
                    .map(|blocks| {
                        blocks
                            .iter()
                            .filter_map(|block| block["text"].as_str())
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .unwrap_or_default();
                if !text.is_empty() {
                    self.note = Some(text.clone());
                }
                let update = if let Some(error) = value["error"].as_str() {
                    AgentUpdate::Failure(format!("Claude reported {error}"))
                } else {
                    let tools = content
                        .map(|blocks| {
                            blocks
                                .iter()
                                .filter_map(|block| block["name"].as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    AgentUpdate::Activity {
                        text: if text.is_empty() {
                            format!("Agent tools: {tools}")
                        } else {
                            text.chars().take(240).collect()
                        },
                        warning: None,
                    }
                };
                updates.push(update);
                AgentUpdate::Batch(updates)
            }
            "result" => {
                if self.finished {
                    return AgentObservation::Invalid;
                }
                self.finished = true;
                if let Some(text) = value["result"].as_str() {
                    self.note = Some(text.into());
                }
                let usage = usage(&value["usage"]);
                let mut cost = estimate(&value);
                if cost.usd.is_none() {
                    let (observed, fallback) = self.observed();
                    if usage.as_ref().zip(observed.as_ref()).is_some_and(|(a, b)| {
                        a.input_tokens == b.input_tokens
                            && a.cached_input_tokens == b.cached_input_tokens
                            && a.cache_write_input_tokens == b.cache_write_input_tokens
                            && a.output_tokens == b.output_tokens
                    }) {
                        cost = fallback;
                    }
                }
                let mut updates = vec![AgentUpdate::UsageSnapshot(usage), AgentUpdate::Cost(cost)];
                let denials = value["permission_denials"].as_array().map_or(0, Vec::len);
                if denials > 0 {
                    updates.push(AgentUpdate::Activity {
                        text: format!("Sandbox refusals: {denials}"),
                        warning: Some(format!(
                            "The sandbox refused {denials} of the agent's commands. Read permission_denials in events.jsonl"
                        )),
                    });
                }
                let aborted = value["terminal_reason"]
                    .as_str()
                    .is_some_and(|reason| reason != "completed");
                if value["subtype"] == "success" && value["is_error"] == false && !aborted {
                    updates.push(AgentUpdate::Turn(None));
                } else {
                    let errors = value["errors"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(Value::as_str)
                                .collect::<Vec<_>>()
                                .join("; ")
                        })
                        .unwrap_or_default();
                    updates.push(AgentUpdate::Failure(format!(
                        "Claude did not complete: {} {errors}",
                        value["subtype"].as_str().unwrap_or("unknown result")
                    )));
                }
                AgentUpdate::Batch(updates)
            }
            _ => AgentUpdate::Unknown,
        };
        AgentObservation::Event {
            kind: kind.into(),
            update,
        }
    }
}
fn usage(value: &Value) -> Option<Usage> {
    let read = value["cache_read_input_tokens"].as_u64()?;
    let write = value["cache_creation_input_tokens"].as_u64()?;
    Some(Usage {
        input_tokens: value["input_tokens"]
            .as_u64()?
            .checked_add(read)?
            .checked_add(write)?,
        cached_input_tokens: read,
        cache_write_input_tokens: Some(write),
        output_tokens: value["output_tokens"].as_u64()?,
        reasoning_output_tokens: None,
    })
}
fn estimate(value: &Value) -> Estimate {
    let usd = value["total_cost_usd"]
        .as_f64()
        .filter(|v| v.is_finite() && *v >= 0.0);
    Estimate {
        usd,
        basis: Basis::ProviderReported,
        models: value["modelUsage"]
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default(),
        source:
            "https://platform.claude.com/docs/en/agent-sdk/cost-tracking#track-cumulative-costs"
                .into(),
        note: if usd.is_some() {
            "API price estimate reported by Claude Code. This is not a subscription charge."
        } else {
            "API cost unavailable: Claude Code did not report a cost."
        }
        .into(),
    }
}
