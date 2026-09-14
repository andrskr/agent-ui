use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Basis {
    Requests,
    RunTotals,
    ProviderReported,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Estimate {
    pub source: String,
    pub usd: Option<f64>,
    pub basis: Basis,
    pub models: Vec<String>,
    pub note: String,
}

pub fn format_usd(usd: Option<f64>) -> String {
    match usd {
        None => "Unavailable".into(),
        Some(value) if value > 0.0 && value < 0.01 => "<$0.01".into(),
        Some(value) => format!("${value:.2}"),
    }
}

pub fn format_difference(usd: Option<f64>) -> String {
    match usd {
        None => "—".into(),
        Some(0.0) => "$0.00".into(),
        Some(value) => format!(
            "{}{}",
            if value < 0.0 { "−" } else { "+" },
            format_usd(Some(value.abs()))
        ),
    }
}

pub fn unavailable(note: &str) -> Estimate {
    Estimate {
        usd: None,
        basis: Basis::RunTotals,
        models: Vec::new(),
        source: String::new(),
        note: note.into(),
    }
}

/// Evidence used to estimate API prices for a run or an assessment.
pub(crate) struct Input<'a> {
    pub model: &'a str,
    pub created_at_ms: u64,
    pub usage: Option<&'a crate::evidence::Usage>,
    pub thread_id: Option<&'a str>,
    pub saved: Option<Estimate>,
}
