use super::super::{Descriptor, Model};

// Model IDs cross-checked with CodexBar a5f2c581ce2e859dab983e28af50c03351db7dd3.
// Prices do not establish CLI access. This catalog describes selectable models.
// Efforts and visible models: Codex models_cache.json, checked 2026-09-14.
const EFFORTS: &[&str] = &["low", "medium", "high", "xhigh", "max"];
const ASTRA_EFFORTS: &[&str] = &["low", "medium", "high", "xhigh", "max", "ultra"];
pub(super) static DESCRIPTOR: Descriptor = Descriptor {
    id: "codex",
    label: "OpenAI / Codex",
    default_model: "gpt-5.6-luna",
    custom_model_efforts: ASTRA_EFFORTS,
    models: &[
        Model {
            id: "gpt-6-astra",
            label: "Astra",
            aliases: &[],
            efforts: ASTRA_EFFORTS,
            default_effort: "medium",
        },
        Model {
            id: "gpt-5.6-luna",
            label: "Luna",
            aliases: &[],
            efforts: EFFORTS,
            default_effort: "low",
        },
        Model {
            id: "gpt-5.6-sol",
            label: "Sol",
            aliases: &["gpt-5.6"],
            efforts: ASTRA_EFFORTS,
            default_effort: "low",
        },
        Model {
            id: "gpt-5.6-terra",
            label: "Terra",
            aliases: &[],
            efforts: ASTRA_EFFORTS,
            default_effort: "medium",
        },
        Model {
            id: "gpt-5.5",
            label: "GPT-5.5",
            aliases: &[],
            efforts: &["low", "medium", "high", "xhigh"],
            default_effort: "medium",
        },
        Model {
            id: "gpt-5.3-codex-spark",
            label: "Codex Spark",
            aliases: &[],
            efforts: &["low", "medium", "high", "xhigh"],
            default_effort: "high",
        },
    ],
    isolation: "Fresh HOME and CODEX_HOME; explicit environment; Codex workspace-write sandbox. Assessment uses read-only. Setup, verification, and preview run on the host.",
};
