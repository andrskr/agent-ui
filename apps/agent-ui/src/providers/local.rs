use crate::{
    storage::{Store, private_dir, valid_run_id},
    toolchain::Tools,
};
use anyhow::Result;
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::Command,
};

/// Private runtime files for one operation. Credentials belong to the provider.
pub(super) struct LocalSession {
    pub root: PathBuf,
    pub home: PathBuf,
    pub env: BTreeMap<String, String>,
    pub binary: PathBuf,
}
impl LocalSession {
    pub fn new(store: &Store, id: &str, app: &Path, tools: &Tools) -> Result<Self> {
        valid_run_id(id)?;
        let root = store.root().join("private").join(id);
        let home = root.join("home");
        private_dir(&home)?;
        let mut session = Self {
            root,
            home,
            env: BTreeMap::new(),
            binary: tools.agent.clone(),
        };
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
    pub fn command(&self, cwd: &Path) -> Command {
        let mut command = Command::new(&self.binary);
        command.env_clear().envs(&self.env).current_dir(cwd);
        command
    }
}
impl Drop for LocalSession {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
