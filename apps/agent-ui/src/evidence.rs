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

impl Usage {
    pub fn add(&mut self, sample: &Self) {
        self.input_tokens += sample.input_tokens;
        self.cached_input_tokens += sample.cached_input_tokens;
        self.output_tokens += sample.output_tokens;
        if let Some(n) = sample.reasoning_output_tokens {
            *self.reasoning_output_tokens.get_or_insert(0) += n;
        }
    }
}
