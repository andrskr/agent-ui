use crate::evidence::{AgentObservation, AgentUpdate, Usage};

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
