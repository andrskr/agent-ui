use anyhow::{Context, Result, ensure};
use std::{env, path::PathBuf, process::Command};

#[derive(Clone)]
pub struct Tools {
    pub agent: PathBuf,
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
            agent: output(Command::new(&self.agent).arg("--version"))?,
            node: output(Command::new(&self.node).arg("--version"))?,
            vp: output(Command::new(&self.vp).arg("--version"))?,
        })
    }
    pub fn discover(settings: &crate::settings::Settings) -> Result<Self> {
        let provider = crate::providers::get(&settings.provider)?;
        let agent = provider.resolve_binary(settings.binary.as_deref())?;
        let node = PathBuf::from(output(
            Command::new(which("node")?).args(["-p", "process.execPath"]),
        )?);
        let rg = provider
            .bundled_rg(&agent)
            .filter(|p| p.is_file())
            .or_else(|| which("rg").ok())
            .context("Cannot find ripgrep. Install ripgrep or check the provider installation")?;
        Ok(Self {
            agent,
            vp: which("vp")?,
            node,
            rg: rg.canonicalize()?,
        })
    }
}
pub struct Versions {
    pub agent: String,
    pub node: String,
    pub vp: String,
}
