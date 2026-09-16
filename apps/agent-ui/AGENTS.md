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
- Read `ARCHITECTURE.md` for ownership rules. Route CLI and TUI actions through `Application`.
- Keep task discovery in `Project`, copied app setup in `PreparedWorkspace`, and saved runs in
  `Store`.
- Keep one current run per task. Validate inputs and tools before deleting the old output. Keep
  replacement, locks, and recovery in `Store`. Do not add history or approval states.
- Use `TaskId` for `<group>--<variant>` names. Keep run ID validation separate. The folder name is
  the only source of group membership. Do not add aliases or old-name compatibility.
- Compare different variants from the same group through `Comparison`. Use the shared group rule for
  CLI actions, the TUI picker, saved selection, and assessments. Keep optional agent assessment
  usage separate. Bind the assessment to exact run IDs. Keep preview ownership per task.
- Keep each provider's models, efforts, commands, login, and pricing in its provider folder.
- Register providers once in `providers/mod.rs`. Do not add provider switches to shared callers.
- Keep provider JSON in `providers/<id>/event.rs`. Feed typed observations to `Report`.
- Use `Journal` for phase timing and report writes. Use the report lifecycle methods for terminal
  states.
- Use the same runner for CLI and TUI actions.
- Preserve raw events. Do not invent usage, progress percentages, or currency costs.
- Keep credentials in private storage outside run evidence and generated projects.
- Keep run output outside this repository. Copy task inputs only when a run starts.
- Save failures and partial reports. Stop the owned process group on cancellation.
- Use task IDs for normal commands. Load optional `task.toml` package settings automatically.
- Keep task configuration limited to exact npm package versions and explicit install-script
  permissions for those versions. Preserve its source, workspace settings, and resolved setup
  lockfile. Apply it only to the run copy, before measuring agent execution.
- Keep tests in memory. Do not create files, change the host environment, start child processes, or
  open browsers or editors from tests. Do not call the run, preview, or editor actions in tests.
- Test observable behavior with independent expected results. Do not test copied logic, display
  labels alone, or mock call counts. Document behavior that needs a separate manual check.
- Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace` from the repository root. Run `vp run agent-ui tasks --check` to validate
  the task catalog without run storage. Run the root verification command.
