mod catalog;
use catalog::DESCRIPTOR;
pub mod cost;
pub mod event;
use super::{Descriptor, Provider, Purpose, Request, Session, local::LocalSession};
use crate::{
    cost::{Estimate, Input},
    evidence::AgentObservation,
    storage::{Store, private_dir},
    toolchain::{Tools, output, which},
};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::BTreeMap,
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) struct Codex;
impl Provider for Codex {
    fn descriptor(&self) -> &'static Descriptor {
        &DESCRIPTOR
    }
    fn resolve_binary(&self, path: Option<&Path>) -> Result<PathBuf> {
        resolve_binary(path)
    }
    fn bundled_rg(&self, binary: &Path) -> Option<PathBuf> {
        Some(binary.parent()?.parent()?.join("codex-path/rg"))
    }
    fn preflight(&self, store: &Store, binary: &Path) -> Result<()> {
        auth_seed(store)?;
        output(
            Command::new(binary)
                .env("CODEX_HOME", store.root().join("private"))
                .args([
                    "-c",
                    "cli_auth_credentials_store=\"file\"",
                    "login",
                    "status",
                ]),
        )?;
        Ok(())
    }
    fn prepare(
        &self,
        store: &Store,
        id: &str,
        tools: &Tools,
        request: &Request<'_>,
    ) -> Result<Box<dyn Session>> {
        let mut local = LocalSession::new(store, id, request.cwd, tools)?;
        let home = local.home.join(".codex");
        private_dir(&home)?;
        let auth = home.join("auth.json");
        fs::copy(auth_seed(store)?, &auth)?;
        fs::set_permissions(auth, fs::Permissions::from_mode(0o600))?;
        fs::write(home.join("config.toml"), CONFIG)?;
        fs::write(request.evidence.join("codex-config.toml"), CONFIG)?;
        local
            .env
            .insert("CODEX_HOME".into(), home.display().to_string());
        Ok(Box::new(CodexSession { local }))
    }
    fn login(&self, store: &Store, binary: &Path) -> Result<()> {
        login(store, binary)
    }
    fn estimate(&self, input: &Input<'_>, evidence: Option<&Path>) -> Estimate {
        if let Some(evidence) = evidence
            && let Some(thread_id) = input.thread_id
            && let Ok(entries) = fs::read_dir(evidence)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if !entry.file_name().to_string_lossy().starts_with("rollout-")
                    || path.extension().is_none_or(|ext| ext != "jsonl")
                {
                    continue;
                }
                let Ok(file) = fs::File::open(path) else {
                    continue;
                };
                if let Some(estimate) =
                    event::request_cost(std::io::BufReader::new(file), thread_id, input.usage)
                {
                    return estimate;
                }
            }
        }
        cost::run_totals(input.model, input.usage, input.created_at_ms)
    }
}
struct CodexSession {
    local: LocalSession,
}
impl Session for CodexSession {
    fn command(&self, request: &Request<'_>) -> Result<Command> {
        let mut command = self.local.command(request.cwd);
        let args = match request.purpose {
            Purpose::Task => exec_args(
                request.settings,
                request.cwd,
                request.note,
                "workspace-write",
            )?,
            Purpose::Assessment => review_args(request.settings, request.cwd, request.note)?,
        };
        command.args(args);
        Ok(command)
    }
    fn environment(&self) -> &BTreeMap<String, String> {
        &self.local.env
    }
    fn decode(&mut self, line: &[u8]) -> AgentObservation {
        event::decode(line)
    }
    fn finish(&self, store: &Store, request: &Request<'_>) -> Result<()> {
        fn visit(path: &Path, target: &Path) -> Result<()> {
            if !path.is_dir() {
                return Ok(());
            }
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    visit(&entry.path(), target)?;
                } else if entry.file_name().to_string_lossy().starts_with("rollout-")
                    && entry.path().extension().is_some_and(|s| s == "jsonl")
                {
                    fs::copy(entry.path(), target.join(entry.file_name()))?;
                }
            }
            Ok(())
        }
        visit(&self.local.home.join(".codex/sessions"), request.evidence)?;
        let auth = self.local.home.join(".codex/auth.json");
        if auth.is_file() {
            fs::copy(auth, store.root().join("private/auth.json"))?;
        }
        Ok(())
    }
}
pub const CONFIG: &str = r#"cli_auth_credentials_store = "file"
web_search = "disabled"
personality = "none"
allow_login_shell = false
shell_environment_policy.inherit = "all"
[features]
apps = false
plugins = false
memories = false
chronicle = false
multi_agent = false
hooks = false
shell_snapshot = false
browser_use = false
computer_use = false
workspace_dependencies = false
skip_host_skill_discovery = true
"#;
fn auth_seed(store: &Store) -> Result<PathBuf> {
    let seed = store.root().join("private/auth.json");
    if !seed.exists() {
        let source = env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or(PathBuf::from(env::var("HOME")?).join(".codex"))
            .join("auth.json");
        ensure!(
            source.is_file(),
            "No file-based Codex login found. Run 'agent-ui login' first"
        );
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&source)?)?;
        ensure!(
            value["tokens"].is_object(),
            "A ChatGPT subscription login is required. Run 'agent-ui login'"
        );
        fs::copy(source, &seed)?;
        fs::set_permissions(&seed, fs::Permissions::from_mode(0o600))?;
    }
    Ok(seed)
}
fn resolve_binary(override_path: Option<&Path>) -> Result<PathBuf> {
    let mut codex = match override_path {
        Some(path) => path.to_path_buf(),
        None => which("codex")?,
    };
    // Vite+ shims need the host home. Find the native Codex file before changing the child environment.
    if override_path.is_none() && codex.canonicalize()?.file_name().is_some_and(|n| n == "vp") {
        let packages = PathBuf::from(env::var("HOME")?).join(".vite-plus/packages/@openai/codex");
        // Vite+ releases store the native file at different folder depths. Search the whole package
        // tree and use the most recent match.
        codex = newest_native_codex(&packages)
            .context("Cannot resolve the Codex shim. Pass --binary with the native binary path")?;
    }
    let codex = codex.canonicalize()?;
    Ok(codex)
}
/// Find the native Codex file below the Vite+ package folder. Return the most recent match.
/// The folder depth changes between Vite+ releases, so this walks the full tree.
fn newest_native_codex(packages: &Path) -> Option<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    let mut stack = vec![packages.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => stack.push(path),
                Ok(kind) if kind.is_file() && is_native_codex(&path) => {
                    let modified = fs::metadata(&path)
                        .and_then(|meta| meta.modified())
                        .unwrap_or(std::time::UNIX_EPOCH);
                    if best.as_ref().is_none_or(|(seen, _)| modified >= *seen) {
                        best = Some((modified, path));
                    }
                }
                _ => {}
            }
        }
    }
    best.map(|(_, path)| path)
}
/// Report if the path is a native Codex file. It sits at `.../vendor/<arch>/bin/codex`.
/// This rejects the Vite+ shim and the helper files next to the native file.
fn is_native_codex(path: &Path) -> bool {
    if path.file_name().is_none_or(|name| name != "codex") {
        return false;
    }
    if path
        .parent()
        .and_then(Path::file_name)
        .is_none_or(|name| name != "bin")
    {
        return false;
    }
    path.ancestors()
        .any(|ancestor| ancestor.file_name().is_some_and(|name| name == "vendor"))
}
pub(crate) fn login(store: &Store, binary: &Path) -> Result<()> {
    let home = store.root().join("private/login");
    private_dir(&home)?;
    let status = Command::new(binary)
        .env("CODEX_HOME", &home)
        .args([
            "-c",
            "cli_auth_credentials_store=\"file\"",
            "login",
            "--device-auth",
        ])
        .status()?;
    if !status.success() {
        bail!("Codex login failed");
    }
    fs::copy(
        home.join("auth.json"),
        store.root().join("private/auth.json"),
    )?;
    fs::set_permissions(
        store.root().join("private/auth.json"),
        fs::Permissions::from_mode(0o600),
    )?;
    fs::remove_dir_all(home)?;
    Ok(())
}
fn exec_args(
    settings: &crate::settings::Settings,
    app: &Path,
    agent_report: &Path,
    sandbox: &str,
) -> Result<Vec<String>> {
    let mut args = vec![
        "-a".into(),
        "never".into(),
        "exec".into(),
        "--json".into(),
        "--color".into(),
        "never".into(),
        "--model".into(),
        settings.model.clone(),
        "--sandbox".into(),
        sandbox.into(),
        "-c".into(),
        format!(
            "model_reasoning_effort={}",
            serde_json::to_string(&settings.effort)?
        ),
        "--cd".into(),
        app.display().to_string(),
        "--output-last-message".into(),
        agent_report.display().to_string(),
        "-".into(),
    ];
    if settings.effort == "default" {
        let index = args.iter().position(|s| s == "-c").unwrap();
        args.drain(index..index + 2);
    }
    Ok(args)
}

pub(crate) fn review_args(
    settings: &crate::settings::Settings,
    cwd: &Path,
    report: &Path,
) -> Result<Vec<String>> {
    let mut args = exec_args(settings, cwd, report, "read-only")?;
    args.insert(args.len() - 1, "--skip-git-repo-check".into());
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn assessment_command_is_read_only_and_does_not_require_a_git_repository() {
        let settings = crate::settings::Settings {
            model: "test".into(),
            effort: "low".into(),
            provider: "codex".into(),
            timeout: 10,
            binary: None,
        };
        let args = review_args(
            &settings,
            Path::new("/assessment"),
            Path::new("/assessment/note.md"),
        )
        .unwrap();
        assert!(args.windows(2).any(|a| a == ["--sandbox", "read-only"]));
        assert!(
            args.windows(2)
                .any(|a| a == ["--output-last-message", "/assessment/note.md"])
        );
        assert!(args.windows(2).any(|a| a == ["--cd", "/assessment"]));
        assert!(args.iter().any(|a| a == "--skip-git-repo-check"));
        assert!(
            !args
                .iter()
                .any(|a| a == "workspace-write" || a.contains("dangerously"))
        );
    }
    #[test]
    fn native_codex_matches_the_vendor_file_at_any_depth() {
        // Layout with no version folder.
        assert!(is_native_codex(Path::new(
            "/home/u/.vite-plus/packages/@openai/codex/lib/node_modules/@openai/codex/node_modules/@openai/codex-darwin-arm64/vendor/aarch64-apple-darwin/bin/codex"
        )));
        // Layout with a version folder.
        assert!(is_native_codex(Path::new(
            "/home/u/.vite-plus/packages/@openai/codex/0.1.13/lib/node_modules/@openai/codex/node_modules/@openai/codex-darwin-arm64/vendor/aarch64-apple-darwin/bin/codex"
        )));
    }
    #[test]
    fn native_codex_rejects_the_shim_and_helper_files() {
        // The Vite+ shim has no vendor ancestor.
        assert!(!is_native_codex(Path::new(
            "/home/u/.vite-plus/packages/@openai/codex/bin/codex"
        )));
        // A helper file next to the native file has a different name.
        assert!(!is_native_codex(Path::new(
            "/home/u/.vite-plus/vendor/aarch64-apple-darwin/bin/codex-code-mode-host"
        )));
        // The bundled shell has a bin parent and a vendor ancestor but the wrong name.
        assert!(!is_native_codex(Path::new(
            "/home/u/.vite-plus/vendor/aarch64-apple-darwin/codex-resources/zsh/bin/zsh"
        )));
    }
}
