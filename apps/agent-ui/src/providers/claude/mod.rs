mod activity;
mod catalog;
use catalog::DESCRIPTOR;
mod cost;
pub mod event;
use super::{Descriptor, Provider, Purpose, Request, Session, local::LocalSession};
use crate::{
    cost::{Estimate, Input},
    evidence::AgentObservation,
    storage::{Store, private_dir, write_json},
    toolchain::{Tools, output, which},
};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) struct Claude;
fn config_dir(store: &Store) -> PathBuf {
    store.root().join("private/providers/claude")
}
fn auth_environment(config: &Path) -> Result<BTreeMap<String, String>> {
    let home = env::var("HOME").context("Claude login needs the macOS HOME")?;
    let user = env::var("USER").context("Claude login needs the macOS USER")?;
    ensure!(
        !home.is_empty() && !user.is_empty(),
        "Claude login needs HOME and USER"
    );
    Ok(identity_environment(config, &home, &user))
}
fn identity_environment(config: &Path, home: &str, user: &str) -> BTreeMap<String, String> {
    // Claude uses HOME and USER to find the macOS Keychain login.
    BTreeMap::from([
        ("HOME".into(), home.into()),
        ("USER".into(), user.into()),
        ("LOGNAME".into(), user.into()),
        ("CLAUDE_CONFIG_DIR".into(), config.display().to_string()),
    ])
}
fn auth_command(binary: &Path, environment: &BTreeMap<String, String>) -> Command {
    let mut command = Command::new(binary);
    command
        .env_clear()
        .envs(environment)
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin");
    command
}
impl Provider for Claude {
    fn descriptor(&self) -> &'static Descriptor {
        &DESCRIPTOR
    }
    fn resolve_binary(&self, path: Option<&Path>) -> Result<PathBuf> {
        path.map(Path::to_path_buf)
            .map_or_else(|| which("claude"), Ok)?
            .canonicalize()
            .context("Cannot resolve the Claude Code binary")
    }
    fn bundled_rg(&self, binary: &Path) -> Option<PathBuf> {
        // Claude Code runs ripgrep from its own binary when it is invoked as `rg`. The sandbox
        // symlinks this path to `rg`, so the symlink name becomes argv0 and the binary acts as rg.
        Some(binary.to_path_buf())
    }
    fn preflight(&self, store: &Store, binary: &Path) -> Result<()> {
        let version = output(Command::new(binary).arg("--version"))?;
        check_version(&version)?;
        let config = config_dir(store);
        ensure!(
            config.is_dir(),
            "Claude login is missing. Run 'agent-ui --provider claude login'"
        );
        let environment = auth_environment(&config)?;
        let status = output(auth_command(binary, &environment).args(["auth", "status"]))
            .context("Claude login is unavailable. Run 'agent-ui --provider claude login'")?;
        check_login(&status)?;
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
        local.env.extend(auth_environment(&config_dir(store))?);
        let settings = runtime_settings(&config_dir(store));
        write_json(&request.evidence.join("claude-settings.json"), &settings)?;
        Ok(Box::new(ClaudeSession {
            local,
            decoder: event::Decoder::new(crate::report::now()),
            activity: activity::Decoder::default(),
            session_id: uuid::Uuid::new_v4().to_string(),
        }))
    }
    fn login(&self, store: &Store, binary: &Path) -> Result<()> {
        let config = config_dir(store);
        private_dir(&config)?;
        let environment = auth_environment(&config)?;
        ensure!(
            auth_command(binary, &environment)
                .args(["auth", "login"])
                .status()?
                .success(),
            "Claude login failed"
        );
        Ok(())
    }
    fn estimate(&self, input: &Input<'_>, _evidence: Option<&Path>) -> Estimate {
        input.saved.clone().unwrap_or_else(|| {
            crate::cost::unavailable("API cost unavailable: Claude Code has not reported a cost.")
        })
    }
}
fn check_login(raw: &str) -> Result<()> {
    let status: serde_json::Value =
        serde_json::from_str(raw).context("Cannot read the Claude login status")?;
    ensure!(
        status["loggedIn"] == true,
        "Claude login is unavailable. Run 'agent-ui --provider claude login'"
    );
    Ok(())
}
fn check_version(raw: &str) -> Result<()> {
    let version = raw
        .split_whitespace()
        .next()
        .and_then(|s| semver::Version::parse(s).ok());
    ensure!(
        version.is_some_and(|v| v >= semver::Version::new(2, 1, 257)),
        "Claude Code 2.1.257 or later is required. Update Claude Code on the run machine"
    );
    Ok(())
}
fn runtime_settings(config: &Path) -> serde_json::Value {
    serde_json::json!({
        "sandbox": {"enabled": true, "autoAllowBashIfSandboxed": true, "allowUnsandboxedCommands": false},
        "permissions": {"deny": [format!("Read(/{}/**)", config.display())]},
        "disableAllHooks": true,
        "autoMemoryEnabled": false,
        "switchModelsOnFlag": false
    })
}
struct ClaudeSession {
    local: LocalSession,
    decoder: event::Decoder,
    activity: activity::Decoder,
    session_id: String,
}
impl Session for ClaudeSession {
    fn command(&self, request: &Request<'_>) -> Result<Command> {
        let mut command = self.local.command(request.cwd);
        command.args(command_args(request, &self.session_id));
        if request.purpose == Purpose::Task {
            let instructions = request.cwd.join("AGENTS.md");
            if instructions.is_file() {
                command.arg("--append-system-prompt-file").arg(instructions);
            }
        }
        Ok(command)
    }
    fn environment(&self) -> &BTreeMap<String, String> {
        &self.local.env
    }
    fn decode(&mut self, line: &[u8]) -> AgentObservation {
        self.decoder.decode(line)
    }
    fn activity(&mut self, line: &[u8]) -> Vec<crate::activity::Event> {
        self.activity.decode(line)
    }
    fn finish(&self, _store: &Store, request: &Request<'_>) -> Result<()> {
        if let Some(note) = &self.decoder.note {
            fs::write(request.note, note)?;
        }
        Ok(())
    }
}
fn command_args(request: &Request<'_>, session_id: &str) -> Vec<String> {
    let mut args: Vec<String> = [
        "--print",
        "--output-format",
        "stream-json",
        "--verbose",
        "--include-partial-messages",
        "--safe-mode",
        "--restricted",
        "--setting-sources",
        "",
        "--no-session-persistence",
        "--model",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    args.extend([
        request.settings.model.clone(),
        "--session-id".into(),
        session_id.into(),
        "--settings".into(),
        request
            .evidence
            .join("claude-settings.json")
            .display()
            .to_string(),
    ]);
    if request.settings.effort != "default" {
        args.extend(["--effort".into(), request.settings.effort.clone()]);
    }
    let tools = match request.purpose {
        Purpose::Task => "Read,Glob,Grep,Edit,Write,Bash",
        Purpose::Assessment => "Read,Glob,Grep",
    };
    args.extend([
        "--tools".into(),
        tools.into(),
        "--permission-mode".into(),
        if request.purpose == Purpose::Task {
            "acceptEdits"
        } else {
            "default"
        }
        .into(),
    ]);
    for path in request.read_roots {
        args.extend(["--add-dir".into(), path.display().to_string()]);
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn login_and_execution_keep_the_keychain_identity_without_inherited_auth_overrides() {
        let identity = identity_environment(
            Path::new("/private/providers/claude"),
            "/Users/tester",
            "tester",
        );
        let command = auth_command(Path::new("/bin/claude"), &identity);
        let environment: BTreeMap<_, _> = command.get_envs().collect();
        assert_eq!(environment.len(), 5);
        for (key, value) in [
            ("HOME", "/Users/tester"),
            ("USER", "tester"),
            ("LOGNAME", "tester"),
            ("CLAUDE_CONFIG_DIR", "/private/providers/claude"),
            ("PATH", "/usr/bin:/bin:/usr/sbin:/sbin"),
        ] {
            assert_eq!(
                environment[std::ffi::OsStr::new(key)],
                Some(std::ffi::OsStr::new(value))
            );
        }
    }
    #[test]
    fn login_status_requires_an_explicit_signed_in_result() {
        assert!(check_login(r#"{"loggedIn":true,"authMethod":"claude.ai"}"#).is_ok());
        for status in [
            r#"{"loggedIn":false}"#,
            r#"{"loggedIn":"true"}"#,
            "{}",
            "Not logged in",
        ] {
            assert!(check_login(status).is_err());
        }
    }
    #[test]
    fn bundled_ripgrep_is_the_claude_binary() {
        assert_eq!(
            Claude.bundled_rg(Path::new("/opt/claude/bin/claude")),
            Some(PathBuf::from("/opt/claude/bin/claude"))
        );
    }
    #[test]
    fn task_and_assessment_commands_keep_distinct_tool_permissions() {
        let settings = crate::settings::Settings::for_provider("claude").unwrap();
        let roots = [PathBuf::from("/runs/one"), PathBuf::from("/runs/two")];
        let mut request = Request {
            settings: &settings,
            cwd: Path::new("/app"),
            evidence: Path::new("/evidence"),
            note: Path::new("/note.md"),
            prompt: "Task",
            purpose: Purpose::Task,
            read_roots: &roots,
        };
        let task = command_args(&request, "session");
        assert!(
            task.windows(2)
                .any(|a| a == ["--tools", "Read,Glob,Grep,Edit,Write,Bash"])
        );
        assert!(
            task.windows(2)
                .any(|a| a == ["--permission-mode", "acceptEdits"])
        );
        request.purpose = Purpose::Assessment;
        let assessment = command_args(&request, "session");
        assert!(
            assessment
                .windows(2)
                .any(|a| a == ["--tools", "Read,Glob,Grep"])
        );
        assert!(
            assessment
                .windows(2)
                .any(|a| a == ["--permission-mode", "default"])
        );
        for args in [&task, &assessment] {
            assert!(args.iter().any(|a| a == "--safe-mode"));
            assert!(args.iter().any(|a| a == "--restricted"));
            assert!(
                args.windows(2)
                    .any(|a| a == ["--output-format", "stream-json"])
            );
            assert!(args.windows(2).any(|a| a == ["--effort", "high"]));
            assert!(args.windows(2).any(|a| a == ["--add-dir", "/runs/two"]));
            assert!(!args.iter().any(|a| a.contains("bypass") || a == "--bare"));
        }
        let mut settings = settings.clone();
        settings.select_model("haiku").unwrap();
        assert_eq!(settings.effort, "default");
        request.settings = &settings;
        assert!(
            !command_args(&request, "session")
                .iter()
                .any(|a| a == "--effort")
        );
    }
    #[test]
    fn settings_require_sandboxed_commands_and_disable_automatic_model_switches() {
        let settings = runtime_settings(Path::new("/private/claude"));
        assert_eq!(settings["sandbox"]["enabled"], true);
        assert_eq!(settings["sandbox"]["allowUnsandboxedCommands"], false);
        assert_eq!(
            settings["permissions"]["deny"][0],
            "Read(//private/claude/**)"
        );
        assert_eq!(settings["switchModelsOnFlag"], false);
        assert!(check_version("2.1.256 (Claude Code)").is_err());
        assert!(check_version("2.1.257 (Claude Code)").is_ok());
        assert!(check_version("3.0.0 (Claude Code)").is_ok());
        assert!(check_version("unknown").is_err());
    }
}
