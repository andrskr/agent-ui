use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct TaskConfig {
    pub dependencies: BTreeMap<String, String>,
    pub dev_dependencies: BTreeMap<String, String>,
    pub allow_builds: BTreeMap<String, bool>,
}

impl TaskConfig {
    pub fn parse(source: &str) -> Result<Self> {
        let config: Self = toml::from_str(source).context("Invalid task.toml")?;
        for (name, version) in config.dependencies.iter().chain(&config.dev_dependencies) {
            let parts: Vec<_> = name.strip_prefix('@').unwrap_or(name).split('/').collect();
            ensure!(
                name.len() <= 214
                    && parts.len() == if name.starts_with('@') { 2 } else { 1 }
                    && parts.iter().all(|part| {
                        part.as_bytes()
                            .first()
                            .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                            && part.bytes().all(|b| {
                                b.is_ascii_lowercase()
                                    || b.is_ascii_digit()
                                    || matches!(b, b'-' | b'_' | b'.')
                            })
                    }),
                "Invalid npm package name '{name}' in task.toml"
            );
            semver::Version::parse(version)
                .with_context(|| format!("Use an exact version for '{name}', such as '1.2.3'; ranges, tags, and paths are not supported"))?;
            ensure!(
                !(config.dependencies.contains_key(name)
                    && config.dev_dependencies.contains_key(name)),
                "Package '{name}' appears in both dependency sections"
            );
        }
        for selector in config.allow_builds.keys() {
            let (name, version) = selector
                .rsplit_once('@')
                .context("Use package@exact-version in allow-builds")?;
            let declared = config
                .dependencies
                .get(name)
                .or_else(|| config.dev_dependencies.get(name));
            ensure!(
                declared.is_some_and(|declared| declared == version),
                "Build permission '{selector}' must match a package and exact version declared in this task"
            );
        }
        Ok(config)
    }

    pub fn has_packages(&self) -> bool {
        !self.dependencies.is_empty() || !self.dev_dependencies.is_empty()
    }

    /// Merge task build permissions into the copied workspace settings.
    pub fn apply_workspace(&self, source: &str) -> Result<String> {
        if self.allow_builds.is_empty() {
            return Ok(source.to_owned());
        }
        let mut workspace: Value =
            serde_saphyr::from_str(source).context("Invalid pnpm-workspace.yaml")?;
        let builds = workspace
            .as_object_mut()
            .context("pnpm-workspace.yaml must be a mapping")?
            .entry("allowBuilds")
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .context("allowBuilds must be a mapping")?;
        for (selector, allowed) in &self.allow_builds {
            builds.insert(selector.clone(), Value::Bool(*allowed));
        }
        serde_saphyr::to_string(&workspace).context("Cannot write workspace settings")
    }

    /// Change only the declared packages. Keep other starter settings.
    pub fn apply(&self, manifest: &Value) -> Result<Value> {
        let mut manifest = manifest.clone();
        let object = manifest
            .as_object_mut()
            .context("package.json must be an object")?;
        for (section, other, packages) in [
            ("dependencies", "devDependencies", &self.dependencies),
            ("devDependencies", "dependencies", &self.dev_dependencies),
        ] {
            if packages.is_empty() {
                continue;
            }
            for (name, version) in packages {
                if let Some(other) = object.get_mut(other) {
                    other
                        .as_object_mut()
                        .context("Dependency sections must be objects")?
                        .remove(name);
                }
                object
                    .entry(section)
                    .or_insert_with(|| serde_json::json!({}))
                    .as_object_mut()
                    .context("Dependency sections must be objects")?
                    .insert(name.clone(), Value::String(version.clone()));
            }
        }
        Ok(manifest)
    }
}
