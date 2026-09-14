use crate::{
    application::Application, artifact::Artifact, process::Cancel, settings::Settings, tui,
};
use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use std::{env, path::PathBuf, thread, time::Duration};

#[derive(Parser)]
#[command(
    version,
    about = "Run local agent experiments and inspect their evidence"
)]
struct Cli {
    #[arg(long, global = true)]
    project: Option<PathBuf>,
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    #[arg(long, global = true)]
    binary: Option<PathBuf>,
    #[arg(long, global = true, default_value = "codex")]
    provider: String,
    #[arg(long, global = true)]
    model: Option<String>,
    #[arg(long, global = true)]
    effort: Option<String>,
    #[arg(long, global = true, default_value_t = 300)]
    timeout: u64,
    #[command(subcommand)]
    command: Option<Action>,
}
#[derive(Subcommand)]
enum Action {
    /// List providers, models, and reasoning efforts.
    Providers,
    /// Open the terminal application.
    Ui {
        #[arg(long)]
        snapshot: bool,
        #[arg(long, default_value_t = 120)]
        width: u16,
        #[arg(long, default_value_t = 36)]
        height: u16,
    },
    /// List available task IDs.
    Tasks,
    /// Replace the task's previous output with a fresh run. Old output is deleted before execution.
    Run { task: String },
    /// Compare the latest results of two tasks. No agent runs unless --assess is set.
    Compare {
        reference: String,
        other: String,
        /// Start a separate read-only agent assessment. This uses subscription tokens.
        #[arg(long, conflicts_with = "saved")]
        assess: bool,
        /// Read the saved assessment for this exact pair. Do not start an agent.
        #[arg(long, conflicts_with = "assess")]
        saved: bool,
    },
    /// Print the task status and its latest JSON report.
    Show { task: String },
    /// Open run files in VS Code.
    Open {
        task: String,
        #[arg(value_enum, default_value = "code")]
        target: Artifact,
    },
    /// Start a dev server until Ctrl+C. Each preview gets a separate port.
    Preview {
        task: String,
        #[arg(long)]
        no_open: bool,
    },
    /// Check local binaries and show storage paths.
    Doctor,
    /// Sign in to the selected provider.
    Login,
}
fn project(cli: &Cli) -> Result<PathBuf> {
    if let Some(path) = &cli.project {
        return Ok(path.clone());
    }
    let cwd = env::current_dir()?;
    cwd.ancestors()
        .find(|p| p.join("experiments/starter/package.json").is_file())
        .map(|p| p.to_path_buf())
        .context("Run from the project, or pass --project")
}
pub fn run() -> Result<()> {
    let mut cli = Cli::parse();
    if matches!(cli.command, Some(Action::Providers)) {
        println!(
            "{}",
            serde_json::to_string_pretty(&crate::providers::catalog())?
        );
        return Ok(());
    }
    let root = cli
        .data_dir
        .clone()
        .unwrap_or(PathBuf::from(env::var("HOME")?).join("Library/Application Support/Agent UI"));
    let mut runtime = Application::open(project(&cli)?, root)?;
    match cli.command.take().unwrap_or(Action::Ui {
        snapshot: false,
        width: 120,
        height: 36,
    }) {
        Action::Providers => unreachable!(),
        Action::Tasks => {
            for task in runtime.task_views()? {
                println!("{:<28} {}", task.id, task.status());
            }
        }
        Action::Show { task } => println!(
            "{}",
            serde_json::to_string_pretty(&runtime.task_view(&task)?)?
        ),
        Action::Compare {
            reference,
            other,
            assess,
            saved,
        } => {
            if assess {
                runtime.start_assessment(&reference, &other, cli.settings()?)?;
                runtime
                    .cancellation()
                    .context("Assessment did not start")?
                    .install_signal_handler()?;
                let report = runtime.join_assessment()?;
                println!("{}", serde_json::to_string_pretty(&report)?);
                println!(
                    "{}",
                    runtime.assessment_text(&runtime.compare(&reference, &other)?)?
                );
                ensure!(
                    report.state == crate::report::State::Ready,
                    "Assessment did not finish successfully"
                );
            } else {
                let comparison = runtime.compare(&reference, &other)?;
                if saved {
                    let text = runtime.assessment_text(&comparison)?;
                    println!(
                        "{}",
                        if text.is_empty() {
                            "No saved assessment for these results."
                        } else {
                            &text
                        }
                    );
                } else {
                    println!("{}", serde_json::to_string_pretty(&comparison)?);
                }
            }
        }
        Action::Open { task, target } => runtime.open_artifact(&task, target)?,
        Action::Preview { task, no_open } => {
            let stop = Cancel::default();
            stop.install_signal_handler()?;
            runtime.start_preview(&task)?;
            loop {
                ensure!(!stop.is_cancelled(), "Preview cancelled during startup");
                let ready = runtime
                    .poll_previews()
                    .into_iter()
                    .find(|(id, _)| id == &task)
                    .context("Preview stopped")?
                    .1?;
                if ready {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
            println!(
                "{}\nPress Ctrl+C to stop the preview.",
                runtime.preview(&task).context("Preview stopped")?.url
            );
            if !no_open {
                runtime.open_preview(&task)?;
            }
            while !stop.is_cancelled() {
                for (_, result) in runtime.poll_previews() {
                    result?;
                }
                thread::sleep(Duration::from_millis(300));
            }
        }
        Action::Doctor => println!("{}", runtime.doctor(&cli.settings()?)?),
        Action::Login => runtime.login(&cli.settings()?)?,
        Action::Ui {
            snapshot,
            width,
            height,
        } => {
            let app = tui::App::new(runtime, cli.settings()?)?;
            if snapshot {
                print!("{}", tui::snapshot(&app, width, height)?);
            } else {
                tui::run(app)?;
            }
        }
        Action::Run { task } => {
            let id = runtime.start(&task, cli.settings()?)?;
            runtime
                .cancellation()
                .context("Run did not start")?
                .install_signal_handler()?;
            eprintln!("Run {}\n{}", id, runtime.run_path(&id)?.display());
            let report = runtime.join(&task)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            ensure!(
                report.state == crate::report::State::Ready,
                "Run ended with {}",
                report.state.label()
            );
        }
    }
    Ok(())
}
impl Cli {
    fn settings(&self) -> Result<Settings> {
        let mut settings = Settings::for_provider(&self.provider)?;
        if let Some(model) = &self.model {
            settings.model = model.clone();
            settings.effort = crate::providers::descriptor(&self.provider)?
                .default_effort(model)
                .into();
        }
        if let Some(effort) = &self.effort {
            settings.effort = effort.clone();
        }
        settings.timeout = self.timeout;
        settings.binary = self.binary.clone();
        settings.validate()?;
        Ok(settings)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_settings_keep_an_unavailable_tool_override_without_resolving_it() {
        let cli = Cli::try_parse_from([
            "agent-ui",
            "--binary",
            "/missing/codex",
            "--model",
            "requested-model",
            "ui",
        ])
        .unwrap();
        let settings = cli.settings().unwrap();
        assert_eq!(settings.model, "requested-model");
        assert_eq!(
            settings.binary.as_deref(),
            Some(std::path::Path::new("/missing/codex"))
        );
        assert!(settings.validate().is_ok());
    }
    #[test]
    fn cli_selects_provider_defaults_and_rejects_incompatible_efforts() {
        let cli = Cli::try_parse_from([
            "agent-ui",
            "--provider",
            "claude",
            "--model",
            "fable",
            "run",
            "smoke",
        ])
        .unwrap();
        let settings = cli.settings().unwrap();
        assert_eq!(
            (&*settings.provider, &*settings.model, &*settings.effort),
            ("claude", "fable", "high")
        );
        let cli = Cli::try_parse_from([
            "agent-ui",
            "--provider",
            "claude",
            "--effort",
            "ultra",
            "run",
            "smoke",
        ])
        .unwrap();
        assert!(cli.settings().is_err());
        let cli =
            Cli::try_parse_from(["agent-ui", "--model", "gpt-6-astra", "run", "smoke"]).unwrap();
        assert_eq!(cli.settings().unwrap().effort, "medium");
    }
}
