use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_output_tokens: Option<u64>,
}

/// Agent observations. The report does not depend on the provider's JSON format.
pub enum AgentObservation {
    Invalid,
    Event { kind: String, update: AgentUpdate },
}

pub enum AgentUpdate {
    Thread(Option<String>),
    Turn(Option<Usage>),
    Failure(String),
    Activity {
        text: String,
        warning: Option<String>,
    },
    Unknown,
}
