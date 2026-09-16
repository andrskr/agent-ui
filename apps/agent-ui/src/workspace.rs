use anyhow::{Result, bail, ensure};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Clone, Copy)]
pub enum CopyMode {
    All,
    Source,
}
pub fn copy_tree(source: &Path, target: &Path, mode: CopyMode) -> Result<()> {
    ensure!(
        !fs::symlink_metadata(source)?.file_type().is_symlink(),
        "Source must not be a link: {}",
        source.display()
    );
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if matches!(mode, CopyMode::Source)
            && matches!(name.to_str(), Some("node_modules" | "dist" | ".git"))
        {
            continue;
        }
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "Input must not be a link: {}",
            entry.path().display()
        );
        if kind.is_dir() {
            copy_tree(&entry.path(), &target.join(name), mode)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), target.join(name))?;
        } else {
            bail!("Input must be a regular file: {}", entry.path().display());
        }
    }
    Ok(())
}
pub fn inventory(root: &Path) -> Result<BTreeMap<String, String>> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<String, String>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if matches!(
                entry.file_name().to_str(),
                Some("node_modules" | "dist" | ".git")
            ) {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_dir() {
                visit(root, &entry.path(), result)?;
            } else if kind.is_file() {
                result.insert(
                    entry
                        .path()
                        .strip_prefix(root)?
                        .to_string_lossy()
                        .into_owned(),
                    format!("{:x}", Sha256::digest(fs::read(entry.path())?)),
                );
            } else if kind.is_symlink() {
                result.insert(
                    entry
                        .path()
                        .strip_prefix(root)?
                        .to_string_lossy()
                        .into_owned(),
                    format!("symlink:{}", fs::read_link(entry.path())?.display()),
                );
            }
        }
        Ok(())
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result)?;
    Ok(result)
}

use crate::{
    journal::Journal,
    project::{TaskInput, TaskSource},
    storage::write_json,
    toolchain::Tools,
};
use std::{path::PathBuf, process::Command, time::Duration};

const TOOLCHAIN: &str = "Notes about this workspace:

- Vite+ manages the toolchain and the package manager. Run `vp <command>`.
- Run `vp check` to format, lint, and type check. Run `vp run verify` to also build.
- You can run the `package.json` scripts with `vp run <name>` or with `pnpm run <name>`.
- Do not install a different package manager. Do not change the toolchain.
- The workspace has no network access and no browser. Check your work with the commands above.";

/// A copied and installed task workspace. Construction saves the setup baseline.
pub(crate) struct PreparedWorkspace {
    pub app: PathBuf,
    pub prompt: String,
}
impl PreparedWorkspace {
    pub fn prepare(source: TaskSource, journal: &mut Journal, tools: &Tools) -> Result<Self> {
        let files = journal.files.clone();
        let evidence = files.evidence();
        fs::create_dir(&evidence)?;
        copy_tree(&source.path, &files.inputs(), CopyMode::All)?;
        let input = TaskInput::load(&files.inputs())?;
        journal.report.inputs = inventory(&files.inputs())?;
        copy_tree(&source.starter, &files.app(), CopyMode::Source)?;
        for name in ["task.md", "AGENTS.md", "references"] {
            let input = files.inputs().join(name);
            if input.is_dir() {
                copy_tree(&input, &files.app().join(name), CopyMode::All)?;
            } else if input.is_file() {
                fs::copy(&input, files.app().join(name))?;
            }
        }
        journal.report.record("Copied the starter and task inputs");
        let config = input.config;
        write_json(&evidence.join("task-config.json"), &config)?;
        journal.report.task_config = Some(config.clone());
        if config.has_packages() {
            let path = files.app().join("package.json");
            let manifest = serde_json::from_slice(&fs::read(&path)?)?;
            write_json(&path, &config.apply(&manifest)?)?;
            if !config.allow_builds.is_empty() {
                let workspace = files.app().join("pnpm-workspace.yaml");
                fs::write(
                    &workspace,
                    config.apply_workspace(&fs::read_to_string(&workspace)?)?,
                )?;
                journal.report.record(format!(
                    "Applied {} task build permissions",
                    config.allow_builds.len()
                ));
            }
            journal.report.record(format!(
                "Applied task packages: {} runtime, {} development",
                config.dependencies.len(),
                config.dev_dependencies.len()
            ));
        }
        journal.step("Installing packages")?;
        let mut install = Command::new(&tools.vp);
        install
            .current_dir(files.app())
            .args([
                "install",
                if config.has_packages() {
                    "--no-frozen-lockfile"
                } else {
                    "--frozen-lockfile"
                },
            ])
            .env("CI", "1");
        journal.command(
            install,
            "Dependency setup",
            "install",
            Duration::from_secs(300),
        )?;
        if config.has_packages() {
            let mut format = Command::new(&tools.vp);
            format
                .current_dir(files.app())
                .args(["fmt", "package.json", "pnpm-workspace.yaml"]);
            journal.command(
                format,
                "Setup formatting",
                "setup-format",
                Duration::from_secs(30),
            )?;
        }
        for (source, target) in [
            ("package.json", "setup-package.json"),
            ("pnpm-lock.yaml", "setup-pnpm-lock.yaml"),
            ("pnpm-workspace.yaml", "setup-pnpm-workspace.yaml"),
        ] {
            fs::copy(files.app().join(source), evidence.join(target))?;
        }
        let mut git = Command::new("git");
        git.current_dir(files.app()).args(["init", "-q"]);
        journal.command(
            git,
            "Workspace Git setup",
            "git-init",
            Duration::from_secs(30),
        )?;
        journal.report.before = inventory(&files.app())?;
        Ok(Self {
            app: files.app(),
            prompt: format!("{}\n\n{TOOLCHAIN}", input.prompt.trim_end()),
        })
    }

    pub fn verify(&self, journal: &mut Journal, tools: &Tools) -> Result<()> {
        journal.report.record("Verification started");
        journal.mark("Verifying the result")?;
        let mut command = Command::new(&tools.vp);
        command
            .current_dir(&self.app)
            .args(["run", "verify"])
            .env("CI", "1");
        journal.command(command, "Verification", "verify", Duration::from_secs(300))?;
        Ok(())
    }
}
