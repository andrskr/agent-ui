use anyhow::{Context, Result, ensure};
use std::{env, path::PathBuf, process::Command};

#[derive(Clone)]
pub struct Tools {
    pub agent: PathBuf,
    pub vp: PathBuf,
    pub node: PathBuf,
    pub pnpm: PathBuf,
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
        let vp = which("vp")?;
        let pnpm = managed_tool_path(&output(Command::new(&vp).args(["exec", "which", "pnpm"]))?)?;
        ensure!(
            pnpm.is_file(),
            "Cannot find managed pnpm: {}",
            pnpm.display()
        );
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
            vp,
            node,
            pnpm,
            rg: rg.canonicalize()?,
        })
    }
}

fn managed_tool_path(output: &str) -> Result<PathBuf> {
    // Vite+ can print its banner before the command output.
    let path = output
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(|line| PathBuf::from(line.trim()))
        .context("Vite+ did not return the managed pnpm path")?;
    ensure!(path.is_absolute(), "Vite+ returned an invalid pnpm path");
    Ok(path)
}

pub struct Versions {
    pub agent: String,
    pub node: String,
    pub vp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_tool_output_accepts_a_banner_but_requires_an_absolute_path() {
        for output in [
            "/tools/pnpm/12.4.1/bin/pnpm\n",
            "VITE+ - The Unified Toolchain for the Web\n\n/tools/pnpm/12.4.1/bin/pnpm\n\n",
        ] {
            assert_eq!(
                managed_tool_path(output).unwrap(),
                PathBuf::from("/tools/pnpm/12.4.1/bin/pnpm")
            );
        }
        for output in [
            "",
            "\n",
            "VITE+ - The Unified Toolchain for the Web\n",
            "pnpm",
        ] {
            assert!(managed_tool_path(output).is_err());
        }
    }
}
