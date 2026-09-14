use crate::{
    cost::{Basis, Estimate},
    evidence::Usage,
};
use std::collections::BTreeSet;

pub const SOURCE: &str = "https://github.com/steipete/CodexBar/blob/a5f2c581ce2e859dab983e28af50c03351db7dd3/Sources/CodexBarCore/Vendored/CostUsage/CostUsagePricing.swift";

pub(super) struct Request {
    pub model: String,
    pub usage: Usage,
    pub cache_write_1h: u64,
    pub timestamp_ms: u64,
}
// Rates and cache rules follow CodexBar. See THIRD_PARTY_NOTICES.md.
fn price(request: &Request) -> Option<f64> {
    let raw = request.model.as_str();
    // A model from another service must not use first-party prices.
    if !raw.starts_with("claude-") || raw.contains('/') {
        return None;
    }
    let model = raw
        .rsplit_once('-')
        .filter(|(_, date)| date.len() == 8 && date.bytes().all(|b| b.is_ascii_digit()))
        .map_or(raw, |(base, _)| base);
    let (mut input, mut output, long) = match model {
        "claude-fable-5" => (10.0, 50.0, false),
        "claude-haiku-4-5" => (1.0, 5.0, false),
        "claude-opus-4-5" | "claude-opus-4-7" | "claude-opus-4-8" => (5.0, 25.0, false),
        "claude-opus-4-6" => (5.0, 25.0, request.timestamp_ms < 1_773_360_000_000),
        "claude-sonnet-4-6" => (3.0, 15.0, request.timestamp_ms < 1_773_360_000_000),
        "claude-sonnet-4" | "claude-sonnet-4-5" => (3.0, 15.0, true),
        "claude-opus-4" | "claude-opus-4-1" => (15.0, 75.0, false),
        _ => return None,
    };
    let usage = &request.usage;
    if long && usage.input_tokens > 200_000 {
        input *= 2.0;
        output *= 1.5;
    }
    let write = usage.cache_write_input_tokens?;
    let new = usage
        .input_tokens
        .checked_sub(usage.cached_input_tokens)?
        .checked_sub(write)?;
    let write_1h = request.cache_write_1h.min(write);
    Some(
        (new as f64 * input
            + usage.cached_input_tokens as f64 * input * 0.1
            + (write - write_1h) as f64 * input * 1.25
            + write_1h as f64 * input * 2.0
            + usage.output_tokens as f64 * output)
            / 1_000_000.0,
    )
}
pub(super) fn estimate<'a>(requests: impl Iterator<Item = &'a Request>) -> Estimate {
    let mut models = BTreeSet::new();
    let mut usd = Some(0.0);
    for request in requests {
        models.insert(request.model.clone());
        usd = usd.zip(price(request)).map(|(sum, price)| sum + price);
    }
    if models.is_empty() {
        usd = None;
    }
    Estimate { usd, basis: Basis::Requests, models: models.into_iter().collect(), source: SOURCE.into(),
        note: if usd.is_some() { "API price estimate from observed Claude messages and CodexBar rates. An unfinished stream can omit usage. This is not a subscription charge." } else { "API cost unavailable: observed Claude messages are incomplete or have an unknown model price." }.into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(model: &str, input: u64) -> Request {
        Request {
            model: model.into(),
            usage: Usage {
                input_tokens: input,
                cached_input_tokens: 60_000,
                cache_write_input_tokens: Some(30_000),
                output_tokens: 10_000,
                reasoning_output_tokens: None,
            },
            cache_write_1h: 10_000,
            timestamp_ms: 1_789_387_200_000,
        }
    }
    #[test]
    fn cache_durations_and_context_thresholds_match_codexbar() {
        assert!((price(&row("claude-sonnet-4-6", 100_000)).unwrap() - 0.333).abs() < 1e-10);
        assert!((price(&row("claude-fable-5", 100_000)).unwrap() - 1.11).abs() < 1e-10);
        assert!((price(&row("claude-sonnet-4-5", 200_000)).unwrap() - 0.633).abs() < 1e-10);
        assert!((price(&row("claude-sonnet-4-5", 200_001)).unwrap() - 1.191006).abs() < 1e-10);
        let mut historical = row("claude-sonnet-4-6", 200_001);
        historical.timestamp_ms = 1_773_359_999_999;
        assert!((price(&historical).unwrap() - 1.191006).abs() < 1e-10);
        historical.timestamp_ms += 1;
        assert!((price(&historical).unwrap() - 0.633003).abs() < 1e-10);
        for model in [
            "claude-sonnet-5",
            "claude-opus-5",
            "claude-fable-5-1",
            "other/claude-sonnet-4-6",
        ] {
            assert!(price(&row(model, 100_000)).is_none());
        }
    }
}
