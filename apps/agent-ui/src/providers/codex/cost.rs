use crate::{
    cost::{Basis, Estimate},
    evidence::Usage,
};

pub use crate::cost::PRICE_SOURCE as SOURCE;

/// One API request. Input includes cache reads and cache writes.
pub struct Request {
    pub model: String,
    pub usage: Usage,
    pub cache_write_input_tokens: u64,
    pub timestamp_ms: u64,
    pub priority: bool,
}

// Rates follow CodexBar with the pinned catalog. See docs/cost-estimation.md.
// Attribution is in THIRD_PARTY_NOTICES.md.
// Values are USD per million tokens.
struct Rates {
    input: f64,
    cached: f64,
    output: f64,
    write: f64,
    threshold: Option<u64>,
}

fn normalize(model: &str) -> &str {
    let model = model.trim();
    let model = model.strip_prefix("openai/").unwrap_or(model);
    match model {
        "gpt-5.6" => "gpt-5.6-sol",
        "gpt-reserve" => "gpt-5.6-luna",
        _ => {
            if model.len() > 11 {
                let split = model.len() - 11;
                if let Some(suffix) = model.get(split..)
                    && suffix.bytes().enumerate().all(|(i, b)| match i {
                        0 | 5 | 8 => b == b'-',
                        _ => b.is_ascii_digit(),
                    })
                {
                    return &model[..split];
                }
            }
            model
        }
    }
}

fn rates(model: &str, timestamp_ms: u64) -> Option<Rates> {
    let historical = timestamp_ms < 1_785_369_600_000;
    let (input, cached, output, write, long_context) = match model {
        "gpt-5" | "gpt-5-codex" | "gpt-5.1" | "gpt-5.1-codex" | "gpt-5.1-codex-max" => {
            (1.25, 0.125, 10.0, 1.25, false)
        }
        "gpt-5-mini" | "gpt-5.1-codex-mini" => (0.25, 0.025, 2.0, 0.25, false),
        "gpt-5-nano" => (0.05, 0.005, 0.4, 0.05, false),
        "gpt-5-pro" => (15.0, 15.0, 120.0, 15.0, false),
        "gpt-5.2" | "gpt-5.2-codex" | "gpt-5.3-codex" => (1.75, 0.175, 14.0, 1.75, false),
        "gpt-5.2-pro" => (21.0, 21.0, 168.0, 21.0, false),
        "gpt-5.3-codex-spark" => (1.75, 0.175, 14.0, 1.75, false),
        "gpt-5.4" => (2.5, 0.25, 15.0, 2.5, true),
        "gpt-5.4-mini" => (0.75, 0.075, 4.5, 0.75, false),
        "gpt-5.4-nano" => (0.2, 0.02, 1.25, 0.2, false),
        "gpt-5.4-pro" | "gpt-5.5-pro" => (30.0, 30.0, 180.0, 30.0, false),
        "gpt-5.5" => (5.0, 0.5, 30.0, 5.0, true),
        "gpt-6-astra" => (10.0, 1.0, 50.0, 12.5, true),
        "gpt-5.6-sol" => (4.0, 0.4, 20.0, 5.0, true),
        "gpt-5.6-terra" if historical => (2.5, 0.25, 15.0, 3.125, true),
        "gpt-5.6-luna" if historical => (1.0, 0.1, 6.0, 1.25, true),
        "gpt-5.6-terra" => (2.0, 0.2, 12.0, 2.5, true),
        "gpt-5.6-luna" => (0.2, 0.02, 1.2, 0.25, true),
        _ => return None,
    };
    Some(Rates {
        input,
        cached,
        output,
        write,
        // CodexBar keeps a bundled threshold when present. Otherwise its catalog
        // parser uses the context_over_200k block, including for Pro models.
        threshold: if long_context {
            Some(272_000)
        } else if matches!(model, "gpt-5.4-pro" | "gpt-5.5-pro") {
            Some(200_000)
        } else {
            None
        },
    })
}

fn calculate(request: &Request, request_context: bool) -> Option<f64> {
    let model = normalize(&request.model);
    let rates = rates(model, request.timestamp_ms)?;
    let input = request.usage.input_tokens;
    // A run total cannot show which requests crossed the context threshold.
    if !request_context && rates.threshold.is_some_and(|limit| input > limit) {
        return None;
    }
    let cached = request.usage.cached_input_tokens.min(input);
    let write = request.cache_write_input_tokens.min(input - cached);
    let long = request_context && rates.threshold.is_some_and(|limit| input > limit);
    let input_multiplier = if long { 2.0 } else { 1.0 };
    let output_multiplier = if long { 1.5 } else { 1.0 };
    let fast = if request.priority && (input <= 272_000 || model == "gpt-6-astra") {
        match model {
            "gpt-5.5" => 2.5,
            "gpt-5.4" | "gpt-5.4-mini" | "gpt-5.6-sol" | "gpt-5.6-terra" | "gpt-5.6-luna"
            | "gpt-6-astra" => 2.0,
            _ => 1.0,
        }
    } else {
        1.0
    };
    Some(
        ((input - cached - write) as f64 * rates.input * input_multiplier
            + cached as f64 * rates.cached * input_multiplier
            + write as f64 * rates.write * input_multiplier
            + request.usage.output_tokens as f64 * rates.output * output_multiplier)
            * fast
            / 1_000_000.0,
    )
}

pub fn requests(requests: &[Request]) -> Estimate {
    let mut models: Vec<_> = requests.iter().map(|r| r.model.clone()).collect();
    models.sort();
    models.dedup();
    let usd = if requests.is_empty() {
        None
    } else {
        requests.iter().map(|r| calculate(r, true)).sum()
    };
    let mut estimate = Estimate {
        source: SOURCE.into(),
        usd,
        basis: Basis::Requests,
        models,
        note: "API price estimate from saved requests. This is not a subscription charge.".into(),
    };
    if usd.is_none() {
        estimate.note = "API cost unavailable: a request has no known model price.".into();
    }
    estimate
}

pub fn run_totals(model: &str, usage: Option<&Usage>, timestamp_ms: u64) -> Estimate {
    let usd = usage.and_then(|usage| {
        calculate(
            &Request {
                model: model.into(),
                usage: usage.clone(),
                cache_write_input_tokens: usage.cache_write_input_tokens.unwrap_or(0),
                timestamp_ms,
                priority: false,
            },
            false,
        )
    });
    Estimate {
        source: SOURCE.into(),
        usd,
        basis: Basis::RunTotals,
        models: vec![model.into()],
        note: match (usage, usd) {
            (None, _) => "API cost unavailable: token usage is not reported.",
            (Some(usage), None) if rates(normalize(model), timestamp_ms).is_some_and(|r| r.threshold.is_some_and(|limit| usage.input_tokens > limit)) => "API cost unavailable: run totals exceed the model's context price threshold. Saved request details are required.",
            (_, None) => "API cost unavailable: the requested model has no known price.",
            _ => "API price estimate from run totals and the requested model. Request details are unavailable. Uses standard rates and reported cache writes; excludes Fast adjustments. This is not a subscription charge.",
        }.into(),
    }
}
