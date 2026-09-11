use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Artifact {
    Code,
    Report,
    Agent,
    Evidence,
}
impl Artifact {
    pub fn file_name(self) -> &'static str {
        match self {
            Self::Code => "app",
            Self::Report => "report.json",
            Self::Agent => "agent-report.md",
            Self::Evidence => "evidence",
        }
    }
}
