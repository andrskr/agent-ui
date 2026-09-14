pub mod claude;
pub mod codex;
mod local;

use crate::{
    cost::{Estimate, Input},
    evidence::AgentObservation,
    process::{self, Cancel, Outcome},
    settings::Settings,
    storage::{Store, write_json},
    toolchain::Tools,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

#[derive(Debug, Serialize)]
pub struct Model {
    pub id: &'static str,
    pub label: &'static str,
    pub aliases: &'static [&'static str],
    pub efforts: &'static [&'static str],
    pub default_effort: &'static str,
}
#[derive(Debug, Serialize)]
pub struct Descriptor {
    pub id: &'static str,
    pub label: &'static str,
    pub models: &'static [Model],
    pub default_model: &'static str,
    pub custom_model_efforts: &'static [&'static str],
    pub isolation: &'static str,
}
impl Descriptor {
    pub fn model(&self, id: &str) -> Option<&Model> {
        self.models
            .iter()
            .find(|m| m.id == id || m.aliases.contains(&id))
    }
    pub fn efforts(&self, model: &str) -> &[&str] {
        self.model(model)
            .map_or(self.custom_model_efforts, |m| m.efforts)
    }
    pub fn default_effort(&self, model: &str) -> &str {
        self.model(model).map_or("default", |m| m.default_effort)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Purpose {
    Task,
    Assessment,
}
pub(crate) struct Request<'a> {
    pub settings: &'a Settings,
    pub cwd: &'a Path,
    pub evidence: &'a Path,
    pub note: &'a Path,
    pub prompt: &'a str,
    pub purpose: Purpose,
    pub read_roots: &'a [PathBuf],
}
pub(crate) trait Provider: Sync {
    fn descriptor(&self) -> &'static Descriptor;
    fn resolve_binary(&self, path: Option<&Path>) -> Result<PathBuf>;
    fn bundled_rg(&self, _binary: &Path) -> Option<PathBuf> {
        None
    }
    fn preflight(&self, store: &Store, binary: &Path) -> Result<()>;
    fn prepare(
        &self,
        store: &Store,
        id: &str,
        tools: &Tools,
        request: &Request<'_>,
    ) -> Result<Box<dyn Session>>;
    fn login(&self, store: &Store, binary: &Path) -> Result<()>;
    fn estimate(&self, input: &Input<'_>, evidence: Option<&Path>) -> Estimate;
}
pub(crate) trait Session: Send {
    fn command(&self, request: &Request<'_>) -> Result<Command>;
    fn environment(&self) -> &BTreeMap<String, String>;
    fn decode(&mut self, line: &[u8]) -> AgentObservation;
    fn finish(&self, store: &Store, request: &Request<'_>) -> Result<()>;
}
// Add each provider once. All shared callers use this registry.
const REGISTRY: &[&dyn Provider] = &[&codex::Codex, &claude::Claude];
pub fn catalog() -> Vec<&'static Descriptor> {
    REGISTRY.iter().map(|p| p.descriptor()).collect()
}
pub fn descriptor(id: &str) -> Result<&'static Descriptor> {
    Ok(get(id)?.descriptor())
}
pub(crate) fn get(id: &str) -> Result<&'static dyn Provider> {
    REGISTRY
        .iter()
        .copied()
        .find(|p| p.descriptor().id == id)
        .with_context(|| {
            format!("Unknown provider '{id}'. Run 'agent-ui providers' to list providers")
        })
}
pub(crate) fn estimate(provider: &str, input: &Input<'_>, evidence: Option<&Path>) -> Estimate {
    match get(provider) {
        Ok(provider) => provider.estimate(input, evidence),
        Err(_) => crate::cost::unavailable("API cost unavailable: this provider is not installed."),
    }
}
pub(crate) struct Execution {
    pub outcome: Result<Outcome>,
    pub artifacts: Result<()>,
}
pub(crate) fn execute(
    session: &mut dyn Session,
    store: &Store,
    request: &Request<'_>,
    cancel: &Cancel,
    mut observe: impl FnMut(Vec<AgentObservation>, f64) -> Result<()>,
) -> Result<Execution> {
    let mut command = session.command(request)?;
    std::fs::write(request.evidence.join("prompt.txt"), request.prompt)?;
    write_json(
        &request.evidence.join("environment.json"),
        session.environment(),
    )?;
    write_json(
        &request.evidence.join("command.json"),
        &serde_json::json!({
            "program": command.get_program(), "args": command.get_args().collect::<Vec<_>>()
        }),
    )?;
    let outcome = process::execute(
        &mut command,
        &request.evidence.join("events.jsonl"),
        &request.evidence.join("agent.stderr.log"),
        Some(request.prompt),
        Duration::from_secs(request.settings.timeout),
        cancel,
        |lines, seconds| {
            observe(
                lines.iter().map(|line| session.decode(line)).collect(),
                seconds,
            )
        },
    );
    let artifacts = session.finish(store, request);
    Ok(Execution { outcome, artifacts })
}
