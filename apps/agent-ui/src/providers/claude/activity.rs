use crate::activity::Event;
use serde_json::{Value, json};

#[derive(Default)]
pub(super) struct Decoder {
    message_id: Option<String>,
}
impl Decoder {
    pub fn decode(&mut self, line: &[u8]) -> Vec<Event> {
        let mut events = decode(line);
        for event in &mut events {
            if event.kind == "message.started" {
                self.message_id.clone_from(&event.message_id);
            }
            if matches!(
                event.kind.as_str(),
                "content.started" | "content.finished" | "message.delta" | "message.finished"
            ) {
                event.message_id.clone_from(&self.message_id);
            }
            if event.kind == "message.finished" {
                self.message_id = None;
            }
        }
        events
    }
}

/// Keep provider fields here. Observed tool requests are not tool execution starts.
pub(super) fn decode(line: &[u8]) -> Vec<Event> {
    let Ok(value) = serde_json::from_slice::<Value>(line) else {
        return vec![Event::new(
            "provider.invalid",
            "agent",
            json!({"bytes":line.len()}),
        )];
    };
    let kind = value["type"].as_str().unwrap_or("unknown");
    let mut events = Vec::new();
    let mut add = |name: &str, message: Option<&str>, tool: Option<&str>, details: Value| {
        let mut event = Event::new(name, "agent", details);
        event.message_id = message.filter(|id| !id.is_empty()).map(str::to_owned);
        event.tool_call_id = tool.filter(|id| !id.is_empty()).map(str::to_owned);
        event.parent_tool_call_id = value["parent_tool_use_id"].as_str().map(str::to_owned);
        event.provider_at_ms = value["timestamp"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.timestamp_millis());
        events.push(event);
    };
    match kind {
        "assistant" => {
            let message = &value["message"];
            let id = message["id"].as_str();
            if !message["usage"].is_null() {
                add(
                    "message.usage",
                    id,
                    None,
                    json!({"request_id":value["requestId"],"model":message["model"],"usage":message["usage"],"basis":"message_snapshot"}),
                );
            }
            if let Some(blocks) = message["content"].as_array() {
                for block in blocks {
                    if block["type"] == "tool_use" {
                        add("tool.requested", id, block["id"].as_str(), block.clone());
                    } else {
                        add("message.content", id, None, block.clone());
                    }
                }
            }
            if !value["error"].is_null() {
                add("provider.error", id, None, json!({"error":value["error"]}));
            }
        }
        "user" => {
            if let Some(blocks) = value["message"]["content"].as_array() {
                for block in blocks {
                    if block["type"] == "tool_result" {
                        add(
                            "tool.result",
                            None,
                            block["tool_use_id"].as_str(),
                            block.clone(),
                        );
                    }
                }
            }
        }
        "stream_event" => {
            let event = &value["event"];
            let name = match event["type"].as_str() {
                Some("message_start") => "message.started",
                Some("message_stop") => "message.finished",
                Some("content_block_start") => "content.started",
                Some("content_block_stop") => "content.finished",
                Some("message_delta") => "message.delta",
                // Original deltas remain in the output chunks. Do not duplicate each token.
                Some("content_block_delta") => return events,
                _ => "provider.event",
            };
            add(
                name,
                event["message"]["id"].as_str(),
                event["content_block"]["id"].as_str(),
                event.clone(),
            );
        }
        "result" => add("provider.result", None, None, value.clone()),
        "system" => add("provider.system", None, None, value.clone()),
        _ => add("provider.event", None, None, value.clone()),
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn links_tool_results_and_keeps_usage_as_a_snapshot() {
        let events = decode(br#"{"type":"assistant","requestId":"r1","parent_tool_use_id":"parent","message":{"id":"m1","model":"model","usage":{"input_tokens":12,"output_tokens":3},"content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"false"}}]}}"#);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, "message.usage");
        assert_eq!(events[0].message_id.as_deref(), Some("m1"));
        assert_eq!(events[0].details["usage"]["input_tokens"], 12);
        assert_eq!(events[1].kind, "tool.requested");
        assert_eq!(events[1].tool_call_id.as_deref(), Some("t1"));
        assert_eq!(events[1].parent_tool_call_id.as_deref(), Some("parent"));
        let result = decode(br#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","is_error":true,"content":"Exit code 1"}]}}"#);
        assert_eq!(result[0].kind, "tool.result");
        assert_eq!(result[0].tool_call_id.as_deref(), Some("t1"));
        assert_eq!(result[0].details["is_error"], true);
    }
    #[test]
    fn stream_boundaries_keep_message_identity_without_calling_tool_input_execution() {
        let mut decoder = Decoder::default();
        decoder.decode(
            br#"{"type":"stream_event","event":{"type":"message_start","message":{"id":"m1"}}}"#,
        );
        let start=decoder.decode(br#"{"type":"stream_event","event":{"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"t1","name":"Read"}}}"#);
        assert_eq!(start[0].kind, "content.started");
        assert_eq!(start[0].message_id.as_deref(), Some("m1"));
        assert_eq!(start[0].tool_call_id.as_deref(), Some("t1"));
        let stop = decoder.decode(br#"{"type":"stream_event","event":{"type":"message_stop"}}"#);
        assert_eq!(stop[0].message_id.as_deref(), Some("m1"));
        let next =
            decoder.decode(br#"{"type":"stream_event","event":{"type":"content_block_stop"}}"#);
        assert!(next[0].message_id.is_none());
    }

    #[test]
    fn preserves_unknown_events_without_inventing_times_or_usage() {
        let events = decode(br#"{"type":"new_event","value":17}"#);
        assert_eq!(events[0].details["value"], 17);
        assert_eq!(events[0].provider_at_ms, None);
        assert_eq!(decode(b"broken")[0].kind, "provider.invalid");
        assert!(
            decode(br#"{"type":"stream_event","event":{"type":"content_block_delta"}}"#).is_empty()
        );
    }
}
