# Agent UI

This Rust crate owns the CLI, terminal UI, run files, and child processes.

## Command discovery

Run commands from the repository root:

```sh
vp run agent-ui --help
vp run agent-ui <command> --help
```

Use explicit CLI commands for agent work. Running `vp run agent-ui` without a command opens the
interactive TUI. Read [README.md](README.md) for the run flow, file structure, and known limits.

## Development

- Keep terminal rendering separate from execution and saved reports.
- Use the same runner for CLI and TUI actions.
- Preserve raw events. Do not invent usage, progress percentages, or currency costs.
- Keep credentials in private storage outside run evidence and generated projects.
- Keep run output outside this repository. Copy task inputs only when a run starts.
- Save failures and partial reports. Stop the owned process group on cancellation.
- Use task IDs for normal commands. Do not implement `task.toml` in this phase.
- Keep tests in memory. Do not create files, change the host environment, start child processes, or
  open browsers or editors from tests. Do not call the run, preview, or editor actions in tests.
- Test observable behavior with independent expected results. Do not test copied logic, display
  labels alone, or mock call counts. Document behavior that needs a separate manual check.
- Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace` from the repository root. Run the root verification command.
