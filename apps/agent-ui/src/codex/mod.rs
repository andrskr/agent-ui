use crate::toolchain::Tools;
pub mod event;
use crate::{journal::Journal, storage::write_json, workspace::PreparedWorkspace};
use std::time::Duration;

pub(crate) fn prepare(journal: &mut Journal, tools: &Tools, app: &Path) -> Result<Session> {
    let session = Session::new(journal.store(), &journal.report.id, app, tools)?;
    let evidence = journal.files.evidence();
    fs::write(evidence.join("codex-config.toml"), CONFIG)?;
    write_json(&evidence.join("environment.json"), &session.environment())?;
    let mut command = session.command(tools, app);
    command.args(["login", "status"]);
    journal.command(command, "Codex login", "login", Duration::from_secs(30))?;
    Ok(session)
}

pub(crate) fn execute(
    session: &Session,
    settings: &crate::settings::Settings,
    tools: &Tools,
    workspace: &PreparedWorkspace,
    journal: &mut Journal,
) -> Result<()> {
    let args = exec_args(settings, &workspace.app, &journal.files.agent_report())?;
    let evidence = journal.files.evidence();
    write_json(
        &evidence.join("command.json"),
        &serde_json::json!({"program": tools.codex, "args": args}),
    )?;
    fs::write(evidence.join("prompt.txt"), &workspace.prompt)?;
    journal.report.record("Codex started");
    let mut command = session.command(tools, &workspace.app);
    command.args(args);
    let cancel = journal.cancellation();
    let result = crate::process::execute(
        &mut command,
        &evidence.join("events.jsonl"),
        &evidence.join("codex.stderr.log"),
        Some(&workspace.prompt),
        Duration::from_secs(settings.timeout),
        &cancel,
        |lines, seconds| {
            journal.agent_progress(seconds, lines.iter().map(|line| event::decode(line)))
        },
    );
    if let Ok(outcome) = &result {
        journal.agent_outcome(outcome);
    }
    journal.save()?;
    crate::process::checked(&result?, "Codex")?;
    journal.report.check_agent_completion()
}
use crate::storage::{Store, private_dir};
use anyhow::{Result, bail, ensure};
use std::{
    collections::BTreeMap,
    env, fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::Command,
};

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
pub(crate) struct Session {
    root: PathBuf,
    home: PathBuf,
    env: BTreeMap<String, String>,
}
impl Session {
    pub fn environment(&self) -> &BTreeMap<String, String> {
        &self.env
    }
    pub fn new(store: &Store, id: &str, app: &Path, tools: &Tools) -> Result<Self> {
        crate::storage::valid_id(id)?;
        let root = store.root().join("private").join(id);
        let home = root.join("home");
        let codex_home = home.join(".codex");
        private_dir(&codex_home)?;
        let mut session = Self {
            root,
            home,
            env: BTreeMap::new(),
        };
        let auth = codex_home.join("auth.json");
        fs::copy(auth_seed(store)?, &auth)?;
        fs::set_permissions(auth, fs::Permissions::from_mode(0o600))?;
        fs::write(codex_home.join("config.toml"), CONFIG)?;
        let bin = session.root.join("bin");
        private_dir(&bin)?;
        symlink(&tools.node, bin.join("node"))?;
        symlink(&tools.rg, bin.join("rg"))?;
        fn quote(path: &Path) -> String {
            format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
        }
        let wrapper = bin.join("vp");
        fs::write(
            &wrapper,
            format!(
                "#!/bin/sh\nexec {} {} \"$@\"\n",
                quote(&tools.node),
                quote(&app.join("node_modules/vite-plus/bin/vp"))
            ),
        )?;
        fs::set_permissions(wrapper, fs::Permissions::from_mode(0o700))?;
        let tmp = session.root.join("tmp");
        private_dir(&tmp)?;
        session.env = BTreeMap::from([
            ("HOME".into(), session.home.display().to_string()),
            ("CODEX_HOME".into(), codex_home.display().to_string()),
            (
                "PATH".into(),
                format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", bin.display()),
            ),
            ("TMPDIR".into(), tmp.display().to_string()),
            ("LANG".into(), "en_US.UTF-8".into()),
            ("SHELL".into(), "/bin/bash".into()),
            ("USER".into(), "experiment".into()),
            ("LOGNAME".into(), "experiment".into()),
            ("CI".into(), "1".into()),
            ("NO_COLOR".into(), "1".into()),
        ]);
        Ok(session)
    }
    pub fn command(&self, tools: &Tools, app: &Path) -> Command {
        let mut command = Command::new(&tools.codex);
        command.env_clear().envs(&self.env).current_dir(app);
        command
    }
    pub fn archive(&self, store: &Store, evidence: &Path) -> Result<()> {
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
        visit(&self.home.join(".codex/sessions"), evidence)?;
        let auth = self.home.join(".codex/auth.json");
        if auth.is_file() {
            fs::copy(auth, store.root().join("private/auth.json"))?;
        }
        Ok(())
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
pub(crate) fn login(store: &Store, tools: &Tools) -> Result<()> {
    let _lock = store.lock()?;
    let home = store.root().join("private/login");
    private_dir(&home)?;
    let status = Command::new(&tools.codex)
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
) -> Result<Vec<String>> {
    Ok(vec![
        "-a".into(),
        "never".into(),
        "exec".into(),
        "--json".into(),
        "--color".into(),
        "never".into(),
        "--model".into(),
        settings.model.clone(),
        "--sandbox".into(),
        "workspace-write".into(),
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
    ])
}
