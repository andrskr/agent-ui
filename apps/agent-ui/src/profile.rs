use crate::task::TaskConfig;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path},
};

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
struct Profile {
    dependencies: BTreeMap<String, String>,
    dev_dependencies: BTreeMap<String, String>,
    allow_builds: BTreeMap<String, bool>,
    files: Vec<FileCopy>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileCopy {
    from: String,
    to: String,
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    replace: bool,
}

/// A complete source snapshot. Setup does not read profile sources again.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct SetupPlan {
    pub task: TaskConfig,
    pub packages: TaskConfig,
    pub files: BTreeMap<String, Vec<u8>>,
    pub manifests: BTreeMap<String, String>,
}

pub(crate) fn relative(path: &str) -> Result<()> {
    ensure!(
        !path.is_empty()
            && path
                .split('/')
                .all(|part| !part.is_empty() && part != "." && part != "..")
            && Path::new(path)
                .components()
                .all(|c| matches!(c, Component::Normal(_))),
        "Use a relative path without parent segments: {path}"
    );
    ensure!(!path.contains('\\'), "Use forward slashes in paths");
    Ok(())
}
fn regular_path(root: &Path, path: &str) -> Result<()> {
    relative(path)?;
    let mut cursor = root.to_path_buf();
    for part in Path::new(path).components() {
        cursor.push(part);
        ensure!(
            !fs::symlink_metadata(&cursor)?.file_type().is_symlink(),
            "Profile source must not be a link: {}",
            cursor.display()
        );
    }
    Ok(())
}

impl SetupPlan {
    pub fn resolve(root: &Path, task: &TaskConfig) -> Result<Self> {
        let mut plan = Self {
            task: task.clone(),
            packages: task.clone(),
            ..Self::default()
        };
        for name in &task.setup.profiles {
            let manifest = format!("experiments/profiles/{name}/profile.toml");
            regular_path(root, &manifest)?;
            let raw = fs::read_to_string(root.join(&manifest))?;
            let profile: Profile =
                toml::from_str(&raw).with_context(|| format!("Invalid profile {name}"))?;
            let packages = TaskConfig {
                dependencies: profile.dependencies,
                dev_dependencies: profile.dev_dependencies,
                allow_builds: profile.allow_builds,
                ..TaskConfig::default()
            };
            // Reuse the same exact-version and install-permission validation as tasks.
            packages.validate()?;
            merge_packages(&mut plan.packages, packages)?;
            plan.manifests.insert(name.clone(), raw);
            for copy in profile.files {
                regular_path(root, &copy.from)?;
                relative(&copy.to)?;
                ensure!(
                    !matches!(
                        copy.to
                            .split('/')
                            .next()
                            .map(str::to_ascii_lowercase)
                            .as_deref(),
                        Some(
                            "node_modules"
                                | ".git"
                                | ".agent-ui"
                                | "task.md"
                                | "agents.md"
                                | "references"
                                | "package.json"
                                | "pnpm-lock.yaml"
                                | "pnpm-workspace.yaml"
                        )
                    ),
                    "Reserved profile destination: {}",
                    copy.to
                );
                for pattern in &copy.exclude {
                    if let Some(suffix) = pattern.strip_prefix("**/*") {
                        ensure!(
                            !suffix.is_empty() && !suffix.contains(['*', '?', '/', '\\']),
                            "Use a file suffix after **/*"
                        );
                    } else {
                        relative(pattern)?;
                        ensure!(
                            !pattern.contains(['*', '?']),
                            "Use a relative path or **/*.suffix in exclusions"
                        );
                    }
                }
                collect(
                    root,
                    &copy.from,
                    &copy.to,
                    "",
                    &copy.exclude,
                    copy.replace,
                    &mut plan.files,
                )?;
            }
        }
        Ok(plan)
    }
}
fn collect(
    root: &Path,
    from: &str,
    to: &str,
    relative_name: &str,
    exclude: &[String],
    replace: bool,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    if !relative_name.is_empty()
        && exclude
            .iter()
            .any(|pattern| excluded(relative_name, pattern))
    {
        return Ok(());
    }
    let source = root.join(from);
    let kind = fs::symlink_metadata(&source)?.file_type();
    if kind.is_dir() {
        for entry in fs::read_dir(&source)? {
            let name = entry?
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("Profile paths must use UTF-8"))?;
            let relative_name = if relative_name.is_empty() {
                name.clone()
            } else {
                format!("{relative_name}/{name}")
            };
            collect(
                root,
                &format!("{from}/{name}"),
                &format!("{to}/{name}"),
                &relative_name,
                exclude,
                replace,
                files,
            )?;
        }
    } else {
        ensure!(
            kind.is_file(),
            "Profile input must be a regular file: {from}"
        );
        let starter = root.join("experiments/starter");
        let destination = starter.join(to);
        ensure!(
            !destination.is_dir(),
            "Profile file replaces a starter directory: {to}"
        );
        let mut parent = destination.parent();
        while let Some(path) = parent {
            if path == starter {
                break;
            }
            ensure!(
                !path.is_file(),
                "Profile directory replaces a starter file: {}",
                path.display()
            );
            parent = path.parent();
        }
        insert_file(
            files,
            to,
            fs::read(source)?,
            root.join("experiments/starter").join(to).exists(),
            replace,
        )?;
    }
    Ok(())
}
fn insert_file(
    files: &mut BTreeMap<String, Vec<u8>>,
    to: &str,
    bytes: Vec<u8>,
    starter_exists: bool,
    replace: bool,
) -> Result<()> {
    ensure!(
        replace || !starter_exists,
        "Profile replaces starter file '{to}' without replace = true"
    );
    ensure!(
        !files.keys().any(|key| key.eq_ignore_ascii_case(to)),
        "Profile destination conflict: {to}"
    );
    ensure!(
        !files.keys().any(|key| key
            .to_ascii_lowercase()
            .starts_with(&format!("{}/", to.to_ascii_lowercase()))
            || to
                .to_ascii_lowercase()
                .starts_with(&format!("{}/", key.to_ascii_lowercase()))),
        "Profile file/folder conflict: {to}"
    );
    files.insert(to.into(), bytes);
    Ok(())
}
fn excluded(path: &str, pattern: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix("**/*") {
        path.ends_with(suffix)
    } else {
        path == pattern || path.starts_with(&format!("{pattern}/"))
    }
}
fn merge_packages(target: &mut TaskConfig, source: TaskConfig) -> Result<()> {
    for (name, version) in source.dependencies {
        ensure!(
            !target.dev_dependencies.contains_key(&name),
            "Dependency section conflict: {name}"
        );
        merge(&mut target.dependencies, name, version)?;
    }
    for (name, version) in source.dev_dependencies {
        ensure!(
            !target.dependencies.contains_key(&name),
            "Dependency section conflict: {name}"
        );
        merge(&mut target.dev_dependencies, name, version)?;
    }
    for (selector, allowed) in source.allow_builds {
        merge(&mut target.allow_builds, selector, allowed)?;
    }
    Ok(())
}
fn merge<T: PartialEq>(target: &mut BTreeMap<String, T>, key: String, value: T) -> Result<()> {
    ensure!(
        target.get(&key).is_none_or(|old| old == &value),
        "Conflicting profile setting: {key}"
    );
    target.insert(key, value);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_copies_need_explicit_replacement_and_cannot_overlap() {
        let mut files = BTreeMap::new();
        assert!(insert_file(&mut files, "vite.config.ts", vec![1], true, false).is_err());
        insert_file(&mut files, "vite.config.ts", vec![1], true, true).unwrap();
        assert!(insert_file(&mut files, "vite.config.ts", vec![2], true, true).is_err());
        assert!(insert_file(&mut files, "VITE.CONFIG.TS", vec![2], false, true).is_err());
        assert!(insert_file(&mut files, "vite.config.ts/child", vec![2], false, false).is_err());
        insert_file(&mut files, "tools/plugin/index.ts", vec![3], false, false).unwrap();
        assert!(insert_file(&mut files, "tools", vec![2], false, false).is_err());
        assert_eq!(files["vite.config.ts"], vec![1]);
        assert!(excluded("src/rules/a.spec.ts", "**/*.spec.ts"));
        assert!(excluded("node_modules/tool/index.js", "node_modules"));
        assert!(!excluded("src/rules/a.ts", "**/*.spec.ts"));
    }
    #[test]
    fn paths_cannot_escape_the_copy_roots() {
        for invalid in [
            "", "/tmp/x", "../x", "a/../b", "./x", "a\\b", "a//b", "a/./b", "a/",
        ] {
            assert!(relative(invalid).is_err(), "{invalid}");
        }
        assert!(relative("tools/plugin/src/index.ts").is_ok());
    }
    #[test]
    fn packages_merge_only_when_versions_and_sections_agree() {
        let mut target = TaskConfig::parse("[dev-dependencies]\na = '1.0.0'").unwrap();
        assert!(
            merge_packages(
                &mut target,
                TaskConfig::parse("[dev-dependencies]\na = '2.0.0'").unwrap()
            )
            .is_err()
        );
        assert!(
            merge_packages(
                &mut target,
                TaskConfig::parse("[dependencies]\na = '1.0.0'").unwrap()
            )
            .is_err()
        );
        merge_packages(
            &mut target,
            TaskConfig::parse("[dev-dependencies]\na = '1.0.0'\nb = '2.0.0'").unwrap(),
        )
        .unwrap();
        assert_eq!(target.dev_dependencies.get("b").unwrap(), "2.0.0");
    }
}
