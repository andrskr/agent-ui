use super::super::{Descriptor, Model};

// Model IDs cross-checked with CodexBar a5f2c581ce2e859dab983e28af50c03351db7dd3.
// Prices do not establish CLI access. This catalog describes selectable models.
// Active IDs: https://platform.claude.com/docs/en/about-claude/model-deprecations
// Efforts: https://code.claude.com/docs/en/model-config#adjust-effort-level
// Checked 2026-09-14.
const EFFORTS: &[&str] = &["low", "medium", "high", "xhigh", "max"];
const OLDER_EFFORTS: &[&str] = &["low", "medium", "high", "max"];
pub(super) static DESCRIPTOR: Descriptor = Descriptor {
    id: "claude",
    label: "Anthropic / Claude Code",
    default_model: "claude-sonnet-5",
    custom_model_efforts: EFFORTS,
    models: &[
        Model {
            id: "claude-sonnet-5",
            label: "Sonnet 5",
            aliases: &["sonnet"],
            efforts: EFFORTS,
            default_effort: "high",
        },
        Model {
            id: "claude-opus-5",
            label: "Opus 5",
            aliases: &["opus"],
            efforts: EFFORTS,
            default_effort: "high",
        },
        Model {
            id: "claude-fable-5-1",
            label: "Fable 5.1",
            aliases: &["fable"],
            efforts: EFFORTS,
            default_effort: "high",
        },
        Model {
            id: "claude-fable-5",
            label: "Fable 5",
            aliases: &[],
            efforts: EFFORTS,
            default_effort: "high",
        },
        Model {
            id: "claude-sonnet-4-6",
            label: "Sonnet 4.6",
            aliases: &[],
            efforts: OLDER_EFFORTS,
            default_effort: "high",
        },
        Model {
            id: "claude-opus-4-6",
            label: "Opus 4.6",
            aliases: &["claude-opus-4-6-20260205"],
            efforts: OLDER_EFFORTS,
            default_effort: "high",
        },
        Model {
            id: "claude-haiku-4-5-20251001",
            label: "Haiku 4.5",
            aliases: &["haiku", "claude-haiku-4-5"],
            efforts: &[],
            default_effort: "default",
        },
        Model {
            id: "claude-opus-4-8",
            label: "Opus 4.8",
            aliases: &[],
            efforts: EFFORTS,
            default_effort: "high",
        },
        Model {
            id: "claude-opus-4-7",
            label: "Opus 4.7",
            aliases: &[],
            efforts: EFFORTS,
            default_effort: "xhigh",
        },
        Model {
            id: "claude-opus-4-5-20251101",
            label: "Opus 4.5",
            aliases: &["claude-opus-4-5"],
            efforts: &[],
            default_effort: "default",
        },
        Model {
            id: "claude-sonnet-4-5-20250929",
            label: "Sonnet 4.5",
            aliases: &["claude-sonnet-4-5"],
            efforts: &[],
            default_effort: "default",
        },
    ],
    isolation: "Fresh HOME; private Claude login directory; safe mode and restricted file tools; sandboxed Bash for tasks. Assessments use file-read tools only. Managed machine policy still applies. Setup, verification, and preview run on the host.",
};
