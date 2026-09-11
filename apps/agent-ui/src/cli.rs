use crate::{
    artifact::Artifact,
    codex::{self, Tools},
    preview::{self, Preview},
    process::Cancel,
    runner::{self, Settings},
    settings::Effort,
    storage::Store,
    tui,
};
use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use std::{env, path::PathBuf, thread, time::Duration};

#[derive(Parser)]
#[command(
    version,
    about = "Run local Codex experiments and inspect their evidence"
)]
struct Cli {
    #[arg(long, global = true)]
    project: Option<PathBuf>,
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    #[arg(long, global = true)]
    codex: Option<PathBuf>,
    #[arg(long, global = true, default_value = "gpt-5.6-luna")]
    model: String,
    #[arg(long, global = true, default_value = "low", value_enum)]
    effort: Effort,
    #[arg(long, global = true, default_value_t = 300)]
    timeout: u64,
    #[command(subcommand)]
    command: Option<Action>,
}
#[derive(Subcommand)]
enum Action {
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
    /// Copy a task and starter, run Codex, and save the report.
    Run { task: String },
    /// List saved runs.
    List,
    /// Print the saved JSON report.
    Show { run: String },
    /// Open run files in VS Code.
    Open {
        run: String,
        #[arg(value_enum, default_value = "code")]
        target: Artifact,
    },
    /// Start a dev server until Ctrl+C. Each preview gets a separate port.
    Preview {
        run: String,
        #[arg(long)]
        no_open: bool,
    },
    /// Remove one run and all its files.
    Remove {
        run: String,
        #[arg(long)]
        yes: bool,
    },
    /// Check local binaries and show storage paths.
    Doctor,
    /// Sign in with a separate ChatGPT subscription login.
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
    let root = cli
        .data_dir
        .clone()
        .unwrap_or(PathBuf::from(env::var("HOME")?).join("Library/Application Support/Agent UI"));
    let store = Store::new(project(&cli)?, root)?;
    store.recover()?;
    match cli.command.take().unwrap_or(Action::Ui {
        snapshot: false,
        width: 120,
        height: 36,
    }) {
        Action::Tasks => {
            for task in store.tasks()? {
                println!("{task}");
            }
        }
        Action::List => {
            for report in store.list()? {
                println!(
                    "{}  {:<12} {}",
                    report.id,
                    report.state.label(),
                    report.task
                );
            }
        }
        Action::Show { run } => println!("{}", serde_json::to_string_pretty(&store.load(&run)?)?),
        Action::Remove { run, yes } => {
            ensure!(yes, "Pass --yes to remove run {run} and all its files");
            store.remove(&run)?;
            println!("Removed {run}");
        }
        Action::Open { run, target } => {
            let path = store.artifact(&run, target)?;
            preview::open_editor(&path)?;
        }
        Action::Preview { run, no_open } => {
            let stop = Cancel::default();
            stop.install_signal_handler()?;
            let mut preview = Preview::start(&store, &run, &codex::which("vp")?)?;
            preview.wait(&stop)?;
            println!("{}\nPress Ctrl+C to stop the preview.", preview.url());
            if !no_open {
                preview::open_url(preview.url())?;
            }
            while !stop.is_cancelled() {
                ensure!(
                    preview.is_running()?,
                    "Preview stopped. See preview.stderr.log"
                );
                thread::sleep(Duration::from_millis(300));
            }
        }
        Action::Doctor => doctor(&store, &Tools::discover(cli.codex.as_deref())?)?,
        Action::Login => codex::login(&store, &Tools::discover(cli.codex.as_deref())?)?,
        Action::Ui {
            snapshot,
            width,
            height,
        } => {
            let app = tui::App::new(store, cli.settings())?;
            if snapshot {
                print!("{}", tui::snapshot(&app, width, height)?);
            } else {
                tui::run(app)?;
            }
        }
        Action::Run { task } => {
            let active = runner::start(store.clone(), &task, cli.settings())?;
            active.cancellation().install_signal_handler()?;
            eprintln!("Run {}\n{}", active.id, store.dir(&active.id)?.display());
            let report = active.join()?;
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
    fn settings(&self) -> Settings {
        Settings {
            model: self.model.clone(),
            effort: self.effort,
            timeout: self.timeout,
            codex: self.codex.clone(),
        }
    }
}
fn doctor(store: &Store, tools: &Tools) -> Result<()> {
    for (name, path) in [
        ("Codex", &tools.codex),
        ("Node", &tools.node),
        ("Vite+", &tools.vp),
        ("Ripgrep", &tools.rg),
    ] {
        println!(
            "{name}: {}",
            codex::output(std::process::Command::new(path).arg("--version"))?
        );
        println!("Path: {}", path.display());
    }
    println!(
        "Project: {}\nRun storage: {}\nTasks: {}",
        store.project().display(),
        store.root().display(),
        store.tasks()?.len()
    );
    println!(
        "Credential file: {}",
        if store.root().join("private/auth.json").is_file() {
            "present"
        } else {
            "imported from Codex on first run, or use login"
        }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_settings_keep_an_unavailable_tool_override_without_resolving_it() {
        let cli = Cli::try_parse_from([
            "agent-ui",
            "--codex",
            "/missing/codex",
            "--model",
            "requested-model",
            "ui",
        ])
        .unwrap();
        let settings = cli.settings();
        assert_eq!(settings.model, "requested-model");
        assert_eq!(
            settings.codex.as_deref(),
            Some(std::path::Path::new("/missing/codex"))
        );
        assert!(settings.validate().is_ok());
    }
}
