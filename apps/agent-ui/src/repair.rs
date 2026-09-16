use crate::{
    journal::Journal,
    process::{self, Cancel},
    profile::CheckDefinition,
    storage::write_json,
    toolchain::Tools,
    workspace::{CopyMode, copy_tree, inventory},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub seconds: f64,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attempt {
    pub check: String,
    pub source: BTreeMap<String, String>,
    pub passed: bool,
    pub commands: Vec<CommandResult>,
    pub error: Option<String>,
}

pub fn require_pass(
    attempts: &[Attempt],
    check: &str,
    source: &BTreeMap<String, String>,
) -> Result<()> {
    let last = attempts
        .last()
        .context("Required repair check was not run")?;
    ensure!(
        last.check == check && last.passed,
        "Required repair check did not pass"
    );
    ensure!(
        &last.source == source,
        "Source changed after the last repair check. Run vp run repair again"
    );
    Ok(())
}

pub(crate) fn install_client(app: &Path) -> Result<()> {
    fs::create_dir(app.join(".agent-ui"))?;
    fs::write(
        app.join(".agent-ui/repair.cjs"),
        include_str!("../assets/repair-client.txt"),
    )?;
    let path = app.join("package.json");
    let mut package: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
    let scripts = package
        .get_mut("scripts")
        .and_then(serde_json::Value::as_object_mut)
        .context("Starter scripts must be an object")?;
    ensure!(
        !scripts.contains_key("repair"),
        "The repair script name is reserved"
    );
    scripts.insert("repair".into(), "node .agent-ui/repair.cjs".into());
    write_json(&path, &package)
}

/// Checks use a separate workspace and runner-owned dependencies. The agent only sends a request.
pub(crate) struct Controller {
    app: PathBuf,
    root: PathBuf,
    tools: Tools,
    check: String,
    definition: CheckDefinition,
    protected: BTreeMap<String, String>,
    seen: Option<String>,
    sequence: usize,
}
impl Controller {
    pub fn prepare(
        app: &Path,
        name: &str,
        definition: CheckDefinition,
        tools: &Tools,
        journal: &mut Journal,
    ) -> Result<Self> {
        ensure!(
            cfg!(target_os = "macos") && Path::new("/usr/bin/sandbox-exec").is_file(),
            "Repair requires the macOS command sandbox"
        );
        let root = journal.files.evidence().join("repair");
        fs::create_dir(&root)?;
        for dir in ["home", "tmp", "bin"] {
            fs::create_dir(root.join(dir))?;
        }
        // Preserve pnpm's relative links, but never refer back to the agent's dependency tree.
        let mut copy = Command::new("/bin/cp");
        copy.args(["-R"])
            .arg(app.join("node_modules"))
            .arg(root.join("node_modules"));
        journal.command(
            copy,
            "Saving check dependencies",
            "repair-dependencies",
            Duration::from_secs(120),
        )?;
        for name in [".vite-temp", ".vite", ".cache"] {
            let cache = root.join("node_modules").join(name);
            if cache.exists() {
                fs::remove_dir_all(&cache)?;
            }
            let temporary = root.join("tmp").join(name);
            fs::create_dir(&temporary)?;
            symlink(temporary, cache)?;
        }
        symlink(&tools.node, root.join("bin/node"))?;
        {
            let (name, executable) = ("pnpm", tools.pnpm.clone());
            let wrapper = root.join("bin").join(name);
            fs::write(
                &wrapper,
                format!("#!/bin/sh\nexec {} \"$@\"\n", shell_quote(&executable)),
            )?;
            fs::set_permissions(wrapper, fs::Permissions::from_mode(0o700))?;
        }
        let wrapper = root.join("bin/vp");
        fs::write(
            &wrapper,
            format!(
                "#!/bin/sh\nexec {} {} \"$@\"\n",
                shell_quote(&tools.node),
                shell_quote(&root.join("node_modules/vite-plus/bin/vp"))
            ),
        )?;
        fs::set_permissions(wrapper, fs::Permissions::from_mode(0o700))?;
        Ok(Self {
            app: app.into(),
            root,
            tools: tools.clone(),
            check: name.into(),
            definition,
            protected: protected(&inventory(app)?),
            seen: None,
            sequence: 0,
        })
    }
    pub fn preflight(&mut self, journal: &mut Journal) -> Result<()> {
        journal.step("Checking the repair setup")?;
        let (attempt, _) = self.run(&journal.cancellation(), Duration::from_secs(300))?;
        write_json(&self.root.join("preflight.json"), &attempt)?;
        ensure!(
            attempt.passed,
            "Repair setup check failed. See evidence/repair/preflight.json and its command logs"
        );
        Ok(())
    }
    pub fn poll(&mut self, journal: &mut Journal, remaining: Duration) -> Result<()> {
        let channel = self.app.join(".agent-ui");
        ensure!(
            fs::symlink_metadata(&channel)?.file_type().is_dir(),
            "Repair channel must be a directory"
        );
        let request = channel.join("request.json");
        let metadata = match fs::symlink_metadata(&request) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            other => other?,
        };
        ensure!(
            metadata.is_file() && metadata.len() < 1024,
            "Invalid repair request file"
        );
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Request {
            id: String,
        }
        let request: Request = serde_json::from_slice(&fs::read(request)?)?;
        ensure!(
            uuid::Uuid::parse_str(&request.id).is_ok(),
            "Invalid repair request ID"
        );
        if self.seen.as_ref() == Some(&request.id) {
            return Ok(());
        }
        self.seen = Some(request.id.clone());
        journal.mark("Running repair checks")?;
        let (attempt, output) = self.run(
            &journal.cancellation(),
            remaining.min(Duration::from_secs(300)),
        )?;
        let passed = attempt.passed;
        journal.report.record(format!(
            "Repair check {}: {}",
            self.check,
            if passed { "passed" } else { "failed" }
        ));
        journal.report.repair_attempts.push(attempt);
        journal.save()?;
        ensure!(
            fs::symlink_metadata(&channel)?.file_type().is_dir(),
            "Repair channel changed during check"
        );
        write_json(
            &channel.join("response.json"),
            &serde_json::json!({"id":request.id, "passed":passed, "output":output}),
        )?;
        Ok(())
    }
    pub fn verify(&mut self, journal: &mut Journal) -> Result<()> {
        journal.step("Verifying the final repair source")?;
        let source = inventory(&self.app)?;
        require_pass(&journal.report.repair_attempts, &self.check, &source)?;
        let (attempt, _) = self.run(&journal.cancellation(), Duration::from_secs(300))?;
        let seconds = attempt.commands.iter().map(|result| result.seconds).sum();
        journal.report.verification = Some(crate::report::Check {
            exit_code: Some(if attempt.passed { 0 } else { 1 }),
            seconds,
        });
        write_json(&self.root.join("verification.json"), &attempt)?;
        ensure!(attempt.passed, "Final repair verification failed");
        require_pass(
            &journal.report.repair_attempts,
            &self.check,
            &inventory(&self.app)?,
        )?;
        // Keep the verified production build with the final source.
        let dist = self.root.join("workspace/dist");
        if dist.is_dir() {
            let target = self.app.join("dist");
            if target.exists() {
                fs::remove_dir_all(&target)?;
            }
            copy_tree(&dist, &target, CopyMode::All)?;
        }
        Ok(())
    }
    fn run(&mut self, cancel: &Cancel, timeout: Duration) -> Result<(Attempt, String)> {
        self.sequence += 1;
        let started = Instant::now();
        let source = inventory(&self.app)?;
        let mut attempt = Attempt {
            check: self.check.clone(),
            source: source.clone(),
            passed: false,
            commands: vec![],
            error: None,
        };
        let mut output = String::new();
        let result = (|| -> Result<()> {
            ensure!(
                protected(&source) == self.protected,
                "Check configuration or setup files changed. Restore them before repair"
            );
            ensure!(
                !source.values().any(|value| value.starts_with("symlink:")),
                "Repair source must not contain links"
            );
            let workspace = self.root.join("workspace");
            if workspace.exists() {
                fs::remove_dir_all(&workspace)?;
            }
            copy_tree(&self.app, &workspace, CopyMode::Source)?;
            ensure!(
                inventory(&workspace)? == source,
                "Source changed while the check snapshot was copied"
            );
            symlink(
                self.root.join("node_modules"),
                workspace.join("node_modules"),
            )?;
            let policy = sandbox_policy(&workspace, &self.root.join("tmp"))?;
            for (index, args) in self.definition.commands.iter().enumerate() {
                ensure!(
                    started.elapsed() < timeout,
                    "Repair check time limit reached"
                );
                let mut command = Command::new("/usr/bin/sandbox-exec");
                command
                    .args(["-p", &policy])
                    .arg(&self.tools.node)
                    .arg(self.root.join("node_modules/vite-plus/bin/vp"))
                    .args(&args[1..]);
                command.current_dir(&workspace).env_clear().envs([
                    ("HOME", self.root.join("home").display().to_string()),
                    (
                        "PATH",
                        format!(
                            "{}:/usr/bin:/bin:/usr/sbin:/sbin",
                            self.root.join("bin").display()
                        ),
                    ),
                    ("TMPDIR", self.root.join("tmp").display().to_string()),
                    ("CI", "1".into()),
                    ("NO_COLOR", "1".into()),
                    ("LANG", "en_US.UTF-8".into()),
                ]);
                let log = self.root.join(format!("{}-{index}.log", self.sequence));
                let stderr = self
                    .root
                    .join(format!("{}-{index}.stderr.log", self.sequence));
                let outcome = process::execute(
                    &mut command,
                    &log,
                    &stderr,
                    None,
                    timeout.saturating_sub(started.elapsed()),
                    cancel,
                    |_, _| Ok(()),
                )?;
                let error = process::checked(&outcome, "Repair check")
                    .err()
                    .map(|error| error.to_string());
                output.push_str(&format!("\n$ {}\n", args.join(" ")));
                output.push_str(&fs::read_to_string(&log)?);
                output.push_str(&fs::read_to_string(&stderr)?);
                attempt.commands.push(CommandResult {
                    command: args.clone(),
                    exit_code: outcome.code,
                    seconds: outcome.seconds,
                    error: error.clone(),
                });
                if outcome.termination != process::Termination::Exited {
                    anyhow::bail!("{}", error.unwrap_or_default());
                }
            }
            ensure!(
                inventory(&workspace)? == source,
                "A check changed the source or configuration"
            );
            ensure!(
                inventory(&self.app)? == source,
                "Source changed during the check. Run repair again"
            );
            ensure!(
                attempt
                    .commands
                    .iter()
                    .all(|command| command.error.is_none()),
                "Checks failed. Fix the reported errors and run vp run repair again"
            );
            Ok(())
        })();
        attempt.passed = result.is_ok();
        attempt.error = result.err().map(|error| format!("{error:#}"));
        if let Some(error) = &attempt.error {
            output.push_str(&format!("\n{error}\n"));
        } else {
            output.push_str("\nRepair check passed for this source.\n");
        }
        write_json(&self.root.join(format!("{}.json", self.sequence)), &attempt)?;
        Ok((attempt, output))
    }
}
impl Drop for Controller {
    fn drop(&mut self) {
        // Logs and source hashes remain. Installed tools and check caches are disposable.
        for name in ["workspace", "node_modules", "tmp", "home", "bin"] {
            let _ = fs::remove_dir_all(self.root.join(name));
        }
    }
}
fn protected(source: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    source
        .iter()
        .filter(|(path, _)| !path.starts_with("src/") && !path.starts_with("public/"))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}
fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}
fn sandbox_policy(workspace: &Path, temporary: &Path) -> Result<String> {
    let workspace = serde_json::to_string(&workspace.canonicalize()?.display().to_string())?;
    let temporary = serde_json::to_string(&temporary.canonicalize()?.display().to_string())?;
    Ok(format!(
        "(version 1)(allow default)(deny network*)(deny file-write*)(allow file-write* (subpath {workspace}) (subpath {temporary}))"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_requires_a_successful_latest_check_for_the_exact_source() {
        let source = BTreeMap::from([("src/app.tsx".into(), "good".into())]);
        assert!(require_pass(&[], "quality", &source).is_err());
        let mut attempt = Attempt {
            check: "quality".into(),
            source: source.clone(),
            passed: true,
            commands: vec![],
            error: None,
        };
        assert!(require_pass(&[attempt.clone()], "quality", &source).is_ok());
        assert!(require_pass(&[attempt.clone()], "other", &source).is_err());
        assert!(require_pass(&[attempt.clone()], "quality", &BTreeMap::new()).is_err());
        attempt.passed = false;
        assert!(require_pass(&[attempt], "quality", &source).is_err());
    }
    #[test]
    fn configuration_is_fixed_but_source_can_change() {
        let mut source = BTreeMap::from([
            ("package.json".into(), "original".into()),
            ("src/app.tsx".into(), "one".into()),
        ]);
        let before = protected(&source);
        source.insert("src/app.tsx".into(), "two".into());
        assert_eq!(protected(&source), before);
        source.insert(".oxlintrc.json".into(), "weaker rules".into());
        assert_ne!(protected(&source), before);
    }
}
