use anyhow::{Context, Result, ensure};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone)]
pub struct Tools {
    pub codex: PathBuf,
    pub vp: PathBuf,
    pub node: PathBuf,
    pub rg: PathBuf,
}
pub fn which(name: &str) -> Result<PathBuf> {
    env::split_paths(&env::var_os("PATH").unwrap_or_default())
        .map(|p| p.join(name))
        .find(|p| p.is_file())
        .with_context(|| format!("Cannot find {name} in PATH"))
}
pub fn output(command: &mut Command) -> Result<String> {
    let result = command.output()?;
    ensure!(
        result.status.success(),
        "Tool failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(String::from_utf8_lossy(&result.stdout).trim().to_owned())
}
impl Tools {
    pub fn versions(&self) -> Result<Versions> {
        Ok(Versions {
            codex: output(Command::new(&self.codex).arg("--version"))?,
            node: output(Command::new(&self.node).arg("--version"))?,
            vp: output(Command::new(&self.vp).arg("--version"))?,
        })
    }
    pub fn discover(codex_override: Option<&Path>) -> Result<Self> {
        let node = PathBuf::from(output(
            Command::new(which("node")?).args(["-p", "process.execPath"]),
        )?);
        let mut codex = match codex_override {
            Some(path) => path.to_path_buf(),
            None => which("codex")?,
        };
        // Vite+ shims need the host home. Find the native Codex file before changing the child environment.
        if codex_override.is_none() && codex.canonicalize()?.file_name().is_some_and(|n| n == "vp")
        {
            let packages =
                PathBuf::from(env::var("HOME")?).join(".vite-plus/packages/@openai/codex");
            let mut candidates = vec![];
            if packages.is_dir() {
                for entry in fs::read_dir(packages)? {
                    let entry = entry?;
                    let modules = entry
                        .path()
                        .join("lib/node_modules/@openai/codex/node_modules/@openai");
                    if !modules.is_dir() {
                        continue;
                    }
                    for package in fs::read_dir(modules)? {
                        let vendor = package?.path().join("vendor");
                        if !vendor.is_dir() {
                            continue;
                        }
                        for arch in fs::read_dir(vendor)? {
                            let path = arch?.path().join("bin/codex");
                            if path.is_file() {
                                candidates.push(path);
                            }
                        }
                    }
                }
            }
            candidates.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());
            codex = candidates.pop().context(
                "Cannot resolve the Codex shim. Pass --codex with the native binary path",
            )?;
        }
        let codex = codex.canonicalize()?;
        let rg = bundled_rg(&codex)
            .filter(|path| path.is_file())
            .or_else(|| which("rg").ok())
            .context("Cannot find ripgrep in the Codex package or PATH. Install ripgrep or reinstall Codex")?;
        Ok(Self {
            codex,
            vp: which("vp")?,
            node,
            rg: rg.canonicalize()?,
        })
    }
}
fn bundled_rg(codex: &Path) -> Option<PathBuf> {
    Some(codex.parent()?.parent()?.join("codex-path/rg"))
}

pub struct Versions {
    pub codex: String,
    pub node: String,
    pub vp: String,
}
