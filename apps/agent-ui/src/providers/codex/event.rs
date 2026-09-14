use crate::evidence::{AgentObservation, AgentUpdate, Usage};

/// Read one fresh session. Incomplete request evidence uses the report totals instead.
pub(crate) fn request_cost(
    reader: impl std::io::BufRead,
    thread_id: &str,
    expected: Option<&Usage>,
) -> Option<crate::cost::Estimate> {
    let mut session_matches = false;
    let mut model = None;
    let mut priority = false;
    let mut previous = [0_u64; 4];
    let mut requests = Vec::new();
    for line in reader.lines() {
        let line = line.ok()?;
        let value: serde_json::Value = serde_json::from_str(&line).ok()?;
        let payload = &value["payload"];
        match value["type"].as_str() {
            Some("session_meta") => {
                session_matches = payload["id"].as_str() == Some(thread_id);
                if !session_matches || payload["forked_from_id"].is_string() {
                    return None;
                }
            }
            Some("turn_context") => {
                model = payload["model"].as_str().map(str::to_owned);
                priority = payload["service_tier"].as_str() == Some("priority");
            }
            Some("event_msg") if payload["type"] == "token_count" => {
                let info = &payload["info"];
                if info.is_null() {
                    continue;
                }
                let total = cost_tokens(&info["total_token_usage"])?;
                if total == previous {
                    continue;
                }
                let last = cost_tokens(&info["last_token_usage"])?;
                // Repeated totals do not add cost. Gaps or resets cannot prove request sizes.
                for i in 0..4 {
                    if total[i].checked_sub(previous[i])? != last[i] {
                        return None;
                    }
                }
                previous = total;
                let timestamp =
                    chrono::DateTime::parse_from_rfc3339(value["timestamp"].as_str()?).ok()?;
                requests.push(crate::providers::codex::cost::Request {
                    model: model.clone().unwrap_or_else(|| "unknown".into()),
                    usage: Usage {
                        input_tokens: last[0],
                        cached_input_tokens: last[1],
                        output_tokens: last[2],
                        reasoning_output_tokens: None,
                        cache_write_input_tokens: None,
                    },
                    cache_write_input_tokens: last[3],
                    timestamp_ms: timestamp.timestamp_millis().try_into().ok()?,
                    priority,
                });
            }
            _ => {}
        }
    }
    if !session_matches || requests.is_empty() {
        return None;
    }
    if let Some(usage) = expected
        && previous[..3]
            != [
                usage.input_tokens,
                usage.cached_input_tokens,
                usage.output_tokens,
            ]
    {
        return None;
    }
    Some(crate::providers::codex::cost::requests(&requests))
}

fn cost_tokens(value: &serde_json::Value) -> Option<[u64; 4]> {
    Some([
        value["input_tokens"].as_u64()?,
        value["cached_input_tokens"]
            .as_u64()
            .or_else(|| value["cache_read_input_tokens"].as_u64())?,
        value["output_tokens"].as_u64()?,
        value["cache_write_input_tokens"].as_u64().unwrap_or(0),
    ])
}

pub fn decode(line: &[u8]) -> AgentObservation {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(line) else {
        return AgentObservation::Invalid;
    };
    let kind = value["type"].as_str().unwrap_or("unknown");
    let update = match kind {
        "thread.started" => AgentUpdate::Thread(value["thread_id"].as_str().map(str::to_owned)),
        "turn.completed" => {
            let usage = &value["usage"];
            let sample = (|| {
                Some(Usage {
                    input_tokens: usage["input_tokens"].as_u64()?,
                    cached_input_tokens: usage["cached_input_tokens"].as_u64()?,
                    output_tokens: usage["output_tokens"].as_u64()?,
                    reasoning_output_tokens: usage["reasoning_output_tokens"].as_u64(),
                    cache_write_input_tokens: usage["cache_write_input_tokens"].as_u64(),
                })
            })();
            AgentUpdate::Turn(sample)
        }
        "turn.failed" | "error" => AgentUpdate::Failure(
            value["error"]["message"]
                .as_str()
                .or(value["message"].as_str())
                .unwrap_or("Codex reported an error")
                .to_owned(),
        ),
        "item.started" | "item.completed" => {
            let item = &value["item"];
            let name = item["type"].as_str().unwrap_or("item");
            let detail = item["command"]
                .as_str()
                .or(item["text"].as_str())
                .or(item["message"].as_str())
                .unwrap_or("");
            AgentUpdate::Activity {
                text: format!(
                    "{} {name}: {}",
                    if kind == "item.started" {
                        "Start"
                    } else {
                        "End"
                    },
                    detail.chars().take(240).collect::<String>()
                ),
                warning: (name == "error").then(|| detail.to_owned()),
            }
        }
        _ => AgentUpdate::Unknown,
    };
    AgentObservation::Event {
        kind: kind.to_owned(),
        update,
    }
}

#[cfg(test)]
mod cost_tests {
    use super::*;
    use serde_json::{Value, json};

    fn tokens(input: u64, cached: u64, output: u64) -> Value {
        json!({"input_tokens": input, "cached_input_tokens": cached, "output_tokens": output})
    }

    fn sample(last: Value, total: Value) -> Value {
        json!({"type": "event_msg", "timestamp": "2026-09-14T10:00:00Z", "payload": {
            "type": "token_count", "info": {"last_token_usage": last, "total_token_usage": total}
        }})
    }

    fn records() -> Vec<Value> {
        vec![
            json!({"type": "session_meta", "payload": {"id": "session"}}),
            json!({"type": "turn_context", "payload": {"model": "gpt-5.6-sol"}}),
            sample(
                tokens(200_000, 100_000, 10_000),
                tokens(200_000, 100_000, 10_000),
            ),
            sample(
                tokens(200_000, 100_000, 10_000),
                tokens(400_000, 200_000, 20_000),
            ),
        ]
    }

    fn estimate(records: &[Value], expected: Option<&Usage>) -> Option<crate::cost::Estimate> {
        let text = records
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        request_cost(text.as_bytes(), "session", expected)
    }

    #[test]
    fn requests_use_observed_models_and_ignore_repeated_totals_and_quota_events() {
        let mut rows = records();
        rows.insert(3, rows[2].clone());
        rows.insert(
            4,
            json!({"type": "event_msg", "payload": {"type": "token_count", "info": null}}),
        );
        let estimate = estimate(&rows, None).unwrap();
        assert!((estimate.usd.unwrap() - 1.7).abs() < 1e-10);
        assert_eq!(estimate.models, ["gpt-5.6-sol"]);
        assert_eq!(estimate.basis, crate::cost::Basis::Requests);
    }

    #[test]
    fn model_changes_and_priority_context_apply_to_the_next_request() {
        let mut rows = records();
        rows.insert(
            3,
            json!({"type": "turn_context", "payload": {
                "model": "gpt-5.6-luna", "service_tier": "priority"
            }}),
        );
        let estimate = estimate(&rows, None).unwrap();
        assert!((estimate.usd.unwrap() - 0.918).abs() < 1e-10);
        assert_eq!(estimate.models, ["gpt-5.6-luna", "gpt-5.6-sol"]);
    }

    #[test]
    fn gaps_resets_wrong_sessions_and_mismatched_totals_use_the_fallback() {
        let mut gap = records();
        gap.remove(2);
        assert!(estimate(&gap, None).is_none());
        let mut reset = records();
        reset.push(sample(tokens(100, 0, 10), tokens(100, 0, 10)));
        assert!(estimate(&reset, None).is_none());
        let mut wrong_session = records();
        wrong_session[0]["payload"]["id"] = json!("other");
        assert!(estimate(&wrong_session, None).is_none());
        let expected = Usage {
            input_tokens: 400_001,
            cached_input_tokens: 200_000,
            output_tokens: 20_000,
            reasoning_output_tokens: None,
            cache_write_input_tokens: None,
        };
        assert!(estimate(&records(), Some(&expected)).is_none());
        assert!(request_cost(b"bad json".as_slice(), "session", None).is_none());
    }

    #[test]
    fn cache_writes_use_their_own_price() {
        let mut rows = records();
        rows.truncate(3);
        rows[2]["payload"]["info"]["last_token_usage"]["cache_write_input_tokens"] = json!(50_000);
        rows[2]["payload"]["info"]["total_token_usage"]["cache_write_input_tokens"] = json!(50_000);
        assert!((estimate(&rows, None).unwrap().usd.unwrap() - 0.9125).abs() < 1e-10);
    }

    #[test]
    fn an_unknown_observed_model_does_not_use_the_requested_model_price() {
        let mut rows = records();
        rows[1]["payload"]["model"] = json!("other/gpt-5.6-sol");
        let estimate = estimate(&rows, None).unwrap();
        assert!(estimate.usd.is_none());
        assert_eq!(estimate.models, ["other/gpt-5.6-sol"]);
    }
}
