use crate::providers;
use anyhow::{Result, ensure};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Settings {
    pub provider: String,
    pub model: String,
    pub effort: String,
    pub timeout: u64,
    pub binary: Option<PathBuf>,
}
impl Default for Settings {
    fn default() -> Self {
        Self::for_provider("codex").expect("The default provider is registered")
    }
}
impl Settings {
    pub fn for_provider(provider: &str) -> Result<Self> {
        let descriptor = providers::descriptor(provider)?;
        Ok(Self {
            provider: provider.into(),
            model: descriptor.default_model.into(),
            effort: descriptor.default_effort(descriptor.default_model).into(),
            timeout: 900,
            binary: None,
        })
    }
    pub fn validate(&self) -> Result<()> {
        let descriptor = providers::descriptor(&self.provider)?;
        ensure!(
            !self.model.is_empty()
                && self.model.len() <= 200
                && self
                    .model
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_./:[]".contains(c)),
            "Use a model ID with letters, numbers, '-', '_', '.', '/', ':', '[' or ']'"
        );
        ensure!(
            self.effort == "default"
                || descriptor
                    .efforts(&self.model)
                    .contains(&self.effort.as_str()),
            "Effort '{}' is not supported for {} / {}. Choices: default, {}",
            self.effort,
            self.provider,
            self.model,
            descriptor.efforts(&self.model).join(", ")
        );
        ensure!(self.timeout > 0, "Time limit must be greater than zero");
        Ok(())
    }
    pub fn select_provider(&mut self, provider: &str) -> Result<()> {
        let next = Self::for_provider(provider)?;
        self.provider = next.provider;
        self.model = next.model;
        self.effort = next.effort;
        self.binary = None;
        Ok(())
    }
    pub fn select_model(&mut self, model: &str) -> Result<()> {
        let descriptor = providers::descriptor(&self.provider)?;
        self.model = model.into();
        if self.effort != "default" && !descriptor.efforts(model).contains(&self.effort.as_str()) {
            self.effort = descriptor.default_effort(model).into();
        }
        Ok(())
    }
    pub fn step_provider(&mut self, delta: isize) {
        let catalog = providers::catalog();
        let i = catalog
            .iter()
            .position(|p| p.id == self.provider)
            .unwrap_or(0);
        let next = (i as isize + delta).rem_euclid(catalog.len() as isize) as usize;
        let _ = self.select_provider(catalog[next].id);
    }
    pub fn step_model(&mut self, delta: isize) {
        let Ok(descriptor) = providers::descriptor(&self.provider) else {
            return;
        };
        let models = descriptor.models;
        if models.is_empty() {
            return;
        }
        let i = models
            .iter()
            .position(|m| m.id == self.model || m.aliases.contains(&self.model.as_str()));
        let next = i.map_or(0, |i| {
            (i as isize + delta).rem_euclid(models.len() as isize) as usize
        });
        let _ = self.select_model(models[next].id);
    }
    pub fn step_effort(&mut self, delta: isize) {
        let Ok(descriptor) = providers::descriptor(&self.provider) else {
            return;
        };
        let choices: Vec<_> = std::iter::once("default")
            .chain(descriptor.efforts(&self.model).iter().copied())
            .collect();
        let i = choices.iter().position(|s| *s == self.effort).unwrap_or(0);
        self.effort =
            choices[(i as isize + delta).rem_euclid(choices.len() as isize) as usize].into();
    }
}
