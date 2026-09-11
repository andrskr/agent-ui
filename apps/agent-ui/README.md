# Agent UI

A local application for coding experiments. The CLI and TUI use the same Rust runner. The terminal
view uses a charcoal background, a mint accent, and separate colours for run states.

The run list groups runs by local creation date. Each item shows the task, state, creation time,
requested model, and effort. Press `/` to search by task, state, model, effort, or run ID. Search
words must all match. Press Enter to keep the search or Esc to clear it. New runs do not move the
current selection.

Click a run, tab, or review button to select it. In the New experiment form, click a field to focus
it. Click its arrows to change the task or effort. The mouse wheel selects runs in the sidebar and
scrolls the selected detail view, including long overview content. Keyboard controls remain
available through `?`. The layout supports terminals of at least 76 columns and 24 rows. It respects
`NO_COLOR`.

The Overview shows one duration breakdown, one token table, verification, changed file paths, and an
excerpt from the saved agent note. File labels mean added (`A`), modified (`M`), deleted (`D`), or
unknown (`?`). The note excerpt uses up to four non-empty lines and 600 characters. Open the full
note with `a`. Missing notes and measurements remain explicit.

Preview and Open code stay visible across Overview, Activity, and Evidence. The preview control
shows startup and ready states. Stop applies to the preview owned by this app. Evidence provides the
JSON report, full agent note, and evidence folder through `r`, `a`, and `f`.

Run from the repository root:

```sh
vp run agent-ui
vp run agent-ui tasks
vp run agent-ui run smoke
vp run agent-ui list
vp run agent-ui show <run-id>
vp run agent-ui open <run-id> code
vp run agent-ui preview <run-id>
vp run agent-ui remove <run-id> --yes
```

Use `--model`, `--effort`, and `--timeout` to set a run. The initial defaults are `gpt-5.6-luna`,
`low`, and 300 seconds. The New experiment view lets you edit the model and effort. The model must
be available to your subscription. Codex errors are saved with the run.

Use `--project <folder>` when you start outside this repository. Use `--data-dir <folder>` to choose
a different storage location. It must be outside the repository.

Building the app needs Cargo. Opening the built UI and inspecting saved runs does not need Codex,
Node, Vite+, Git, or ripgrep. Starting a run resolves these tools before creating run files. It uses
the ripgrep bundled with Codex when available, then checks PATH. Preview needs Vite+ only. The app
currently targets macOS. The editor action uses Visual Studio Code. Browser actions use the default
browser. Run `vp run agent-ui doctor` to check the local paths. Use `--codex <native-binary>` if the
app cannot resolve your Codex launcher. Optional `task.toml` files set task packages.

## Run flow

1. Select a task folder by ID. Folders that start with `_` are templates.
2. Press Enter or click Run experiment. The app locks run execution in this storage location.
3. Copy the starter and task inputs. Apply task packages, when set, and install dependencies.
4. Start Codex with the exact task prompt and a fresh configuration.
5. Save events as they arrive. Show reported usage after each completed turn.
6. Run the generated project's `verify` command. Save its output and exit code.
7. Open the code or start a dev server for human review.

`Ready` means Codex completed and verification passed. It does not mean the UI is approved. There is
no automatic visual judge, score, repair loop, or comparison service.

## Files and ownership

```text
apps/agent-ui/src/
  main.rs          Application entry point
  cli.rs           Command parsing and output
  application.rs   Shared run, preview, review, and removal service
  project.rs       Read-only task catalog and input validation
  task.rs          Pure TOML and package-setting rules
  workspace.rs     Task snapshot, copied app, install, and source inventory
  runner.rs        Worker ownership and the three run phases
  journal.rs       Phase timing, command logs, and saved report writes
  report.rs        Report format, observations, and lifecycle rules
  evidence.rs      Typed agent observations and usage
  codex/
    mod.rs         Codex session, credentials, command, and trace capture
    event.rs       Codex JSON decoding
  toolchain.rs     Local executable discovery and versions
  storage.rs       Saved runs, file paths, locks, recovery, and removal
  process.rs       Child groups, cancellation, prompt input, and log capture
  preview.rs       One preview process and its readiness state
  artifact.rs      Review targets
  settings.rs      Model, effort, and timeout settings
  tui/
    state.rs       Forms, selection, and calls to the application service
    history.rs     Search, stable selection, and date groups
    details.rs     Report content and note excerpts
    input.rs       Keyboard and mouse input
    layout.rs      Rectangles shared by rendering and mouse input
    view.rs        Rendering from app state
    theme.rs       Terminal colours
    mod.rs         Terminal event loop and restoration

~/Library/Application Support/Agent UI/
  active.lock
  private/                 Credentials; never part of run evidence
  runs/<run-id>/
    report.json            Measured result; updated with an atomic rename
    inputs/                Original task.md, optional AGENTS.md, and references
    app/                   The generated project; open this in your editor
    agent-report.md        The final Codex response, if one was produced
    evidence/
      prompt.txt
      command.json
      environment.json
      codex-config.toml
      events.jsonl
      codex.stderr.log
      task-config.json       Parsed task settings
      setup-package.json     Manifest before agent execution
      setup-pnpm-lock.yaml   Resolved dependency versions before agent execution
      setup-pnpm-workspace.yaml   Workspace settings used for this run
      setup-format.log       Formatting of generated setup files
      install.log
      install.stderr.log
      login.log
      login.stderr.log
      verify.log
      verify.stderr.log
      rollout-*.jsonl       Codex session traces, when available
```

The application service owns the active worker and preview. Both CLI and TUI actions use it. The TUI
stores presentation state only; it does not read run files or create child processes. The project
catalog reads inputs. The run store owns saved reports and locks. Task configuration rules operate
on values in memory.

The runner coordinates setup, agent execution, and verification. Setup returns a prepared workspace.
The journal measures phases and saves partial results, including early setup failures. The report
checks phase transitions and requires successful agent completion and verification before `Ready`.
Codex owns wire decoding; the report accepts typed observations. A completed run cannot restart
through the lifecycle API.

Dropping an active worker requests cancellation and waits for it. The process owner stops its child
group before it joins the prompt writer, including when an event callback fails. One preview owner
handles startup, readiness, and shutdown. Run removal stops an owned preview before taking the
storage locks. Read [ARCHITECTURE.md](ARCHITECTURE.md) for the assessment and ownership rules.

The saved report remains version 1. This refactor does not require a run-data migration. The app
keeps one synchronous worker and the terminal event loop.

The app copies `task.md`, optional `AGENTS.md`, and `references/` to the project root. It also
preserves the full task folder in `inputs/`. The baseline starter has Astryx packages and no Astryx
CLI. Package installation is a setup phase. The runner does not add agent instructions.

## Task configuration

```text
experiments/tasks/workspace-settings/
  task.md
  task.toml       Optional package settings
  AGENTS.md       Optional project instructions
  references/    Optional input assets
```

For a task that needs the Astryx CLI, use:

```toml
[dev-dependencies]
"@astryxdesign/cli" = "0.5.4"

[allow-builds]
"@astryxdesign/cli@0.5.4" = true
```

Use `[dependencies]` for runtime packages. Both sections accept npm registry names and exact
versions. Unknown fields, duplicate entries across sections, ranges, tags, URLs, and local paths
produce an error before tool discovery or run creation. Package settings can add a dependency,
replace its version, or move it between runtime and development sections. Other starter settings
remain unchanged.

Run the task by ID, or select it in the TUI. No configuration flag is needed. Missing or empty
configuration uses the starter's frozen lockfile. Task packages require a lockfile update in the run
copy. The original starter remains unchanged. Build permissions stay unchanged unless the task
declares `[allow-builds]`. Each key must match a package and exact version declared in this task.
Use `true` to allow its install scripts or `false` to block them. No wildcard or range is accepted.
This first version does not configure transitive package scripts. Settings apply only to the copied
`pnpm-workspace.yaml`.

Setup formats the generated package and workspace files before the source baseline is captured. The
Overview shows declared task packages and build permissions. The report stores `task_config`. Raw
input, parsed configuration, setup manifest, and setup lockfile remain in the run evidence. The
source baseline is captured after setup, so these package changes are not counted as agent edits.
Newly resolved transitive dependencies can differ between runs; compare the saved setup lockfiles
when needed. Older reports load with `task_config: null`, which means no configuration record was
saved.

The commented example in `experiments/tasks/_template/task.toml` leaves the baseline unchanged.

Generated folders (`node_modules`, `dist`, `.git`) are omitted from starter copies and source
inventories. Inputs must contain regular files and folders. Input links are rejected. Input hashes
and before/after source hashes use SHA-256. The report includes added, changed, and deleted source
paths. Dependency contents are not hashed.

## Evidence and limits

The report separates setup, agent execution, and verification duration. Times use a monotonic clock.
The report also records wall-clock timestamps, binary versions, requested model and effort, Codex
thread ID, exit codes, input hashes, source changes, and event counts.

Input tokens include cached input. Reasoning tokens, when supplied, are part of output tokens.
Missing usage is `null`, not zero. Dollar cost is `null`: subscription CLI events do not give a
per-run charge. The requested model is not a claim about the model actually served. Raw rollouts are
kept for further analysis. Their internal format can change across Codex versions.

Cancellation stops the owned process group. Quitting stops the active run and preview. Reports
remain on disk. On restart, an unfinished report without an active storage lock is marked
`interrupted`. Runs do not resume automatically. Only one run executes per storage location. Each
preview uses a separate local port and stops when its owning app or CLI command closes. Removal
needs an explicit confirmation and deletes only the selected run directory.

Codex gets a fresh HOME and CODEX_HOME, an explicit environment, disabled external integrations, and
the workspace-write sandbox. The app copies the existing file-based ChatGPT login into private
storage on the first run. It does not copy global Codex configuration or write back to global auth.
If no usable login exists, run `vp run agent-ui login` to sign in with a separate device flow. Only
the private credential seed keeps refreshes. Per-run private state is removed after trace capture.

This is configuration separation, not full host isolation. Codex still uses the same macOS account.
Host reads, installed binaries, system policy, account limits, and provider caching can be shared.
Dependency setup, verification, and preview execute project code on the host. Use trusted tasks. Raw
logs and traces can contain task content and paths. Review them before sharing them.

The process integration uses the
[Codex JSON event stream](https://learn.chatgpt.com/docs/non-interactive-mode). The environment uses
a separate [Codex home](https://learn.chatgpt.com/docs/config-file/config-advanced).

## Checks

```sh
vp run verify:agent-ui
vp run verify
vp -C experiments/starter run verify
```

The Rust tests run in memory. They check log records split across reads, token totals, missing
usage, completion rules, source changes, ID validation, and process-result classification. UI tests
check search, stable selection, small viewports, scroll extent, preview states, and mouse targets.
They render into Ratatui memory buffers. Task tests parse TOML and apply package settings to JSON in
memory. They do not install packages or run Codex. Expected results come from fixed examples. The
tests do not create files, start child processes, change the host environment, or open browsers or
editors. There are no temporary project fixtures or shell test programs.

These tests do not check real file copies, locks, process cleanup, authentication, or browser and
editor integration. Those paths need a separate manual check when requested. The smoke task starts
real Codex and writes run files. It is not part of the test suite. Earlier manual checks are not
proof that these paths still work after a later change.

The TUI uses Chrono for local date conversion. Ratatui remains pinned to 0.30.2. Its
`unstable-rendered-line-info` feature supplies the wrapped line count used for scrolling and mouse
targets. Review this feature when updating Ratatui.

The `verify:agent-ui` task runs all format, Clippy, and Rust test checks without Vite+ caching.
Cargo and web build commands still write normal build output and tool caches. The no-file rule
applies to test execution, not compilation.

The installed Codex version marks `skip_host_skill_discovery` as under development. The app enables
it to prevent host skill discovery. Its warning remains visible in activity and raw events. Treat
configuration separation as a tested local setup, not a stable guarantee across Codex versions.
