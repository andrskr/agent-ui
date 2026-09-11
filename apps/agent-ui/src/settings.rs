use crate::codex::Tools;
use anyhow::{Result, ensure};
use clap::ValueEnum;
use serde::Serialize;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    #[default]
    Low,
    Medium,
    High,
    Xhigh,
}
impl Effort {
    pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Xhigh];
    pub fn step(self, delta: isize) -> Self {
        let index = match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::Xhigh => 3,
        };
        Self::ALL[(index as isize + delta).rem_euclid(Self::ALL.len() as isize) as usize]
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        }
    }
}
impl fmt::Display for Effort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone)]
pub struct Settings {
    pub model: String,
    pub effort: Effort,
    pub timeout: u64,
    pub tools: Tools,
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        ensure!(!self.model.trim().is_empty(), "Model ID is empty");
        ensure!(self.timeout > 0, "Time limit must be greater than zero");
        Ok(())
    }
}
