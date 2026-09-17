fn main() -> std::process::ExitCode {
    match agent_ui::cli::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            if let Some(code) = error.downcast_ref::<agent_ui::cli::ExitCode>() {
                std::process::ExitCode::from(code.0)
            } else {
                eprintln!("Error: {error:#}");
                std::process::ExitCode::FAILURE
            }
        }
    }
}
