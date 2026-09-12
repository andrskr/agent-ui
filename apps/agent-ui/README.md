# Agent UI

A local application for coding experiments. The CLI and TUI use the same Rust runner. The terminal
view uses a charcoal background, a mint accent, and separate colours for run states.

The sidebar lists tasks by ID. Each item shows its current state and the date and time of its latest
run. Press `/` to search by task or state. Search words must all match. Selection stays on the task
when its run changes. Tasks with no run remain visible.

Click a task, tab, or review button to select it. Use `n` to run the selected task and `c` to
compare it with another task. The mouse wheel selects tasks in the sidebar and scrolls the detail
view. Keyboard controls remain available through `?`. The layout needs at least 76 columns and 24
rows. It respects `NO_COLOR`.

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
vp run agent-ui show smoke
vp run agent-ui open smoke code
vp run agent-ui preview smoke
vp run agent-ui compare <task-a> <task-b>
vp run agent-ui compare <task-a> <task-b> --assess
vp run agent-ui compare <task-a> <task-b> --saved
```

Use `--model`, `--effort`, and `--timeout` to set a run. The initial defaults are `gpt-5.6-luna`,
`low`, and 300 seconds. The Run task view lets you edit the model and effort. The model must be
available to your subscription. Codex errors are saved with the run.

Use `--project <folder>` when you start outside this repository. Use `--data-dir <folder>` to choose
a different storage location. It must be outside the repository.

Building the app needs Cargo. Opening the built UI and inspecting saved runs does not need Codex,
Node, Vite+, Git, or ripgrep. Starting a run resolves these tools before creating run files. It uses
the ripgrep bundled with Codex when available, then checks PATH. Preview needs Vite+ only. The app
currently targets macOS. The editor action uses Visual Studio Code. Browser actions use the default
browser. Run `vp run agent-ui doctor` to check the local paths. Use `--codex <native-binary>` if the
app cannot resolve your Codex launcher. Optional `task.toml` files set task packages.

## Tasks and comparison

The sidebar lists tasks. Each task keeps one run. Select a task to see its latest result or its
prompt if it has no result. Press `n` to start it. The Run form lets you set model and effort.

Starting a run removes that task's previous code, logs, and reports. The app first validates inputs,
settings, and tools. It then takes the execution lock and stops that task's owned preview. It
removes any assessment that uses the old run. It deletes the old output before it creates the new
run. A failed or cancelled new run remains the current result. There is no rollback or run history.

If removal fails, the app shows `Cleanup blocked`. No new run starts. Restart the app or run that
task again to retry cleanup. A preview owned by another process blocks removal. Stop that preview
first. The app does not stop previews for other tasks.

Press `c` to select a second task. Both tasks must have a saved run. Tasks with no run remain
visible in the picker, with a reason why they cannot be selected. The selected task is side A. The
other task is side B. Use `s` to swap them, `v` to choose a side, and `Esc` to return to the task
view. The app saves the selected task and comparison pair.

Overview shows time and token differences as **B minus A**. It also shows model, effort, tool
versions, verification, and saved input, setup, and final source changes. Activity and Evidence show
the selected side. Code and preview actions also use the selected side. Both previews can remain
open at the same time. The comparison reads saved input hashes and reports. Editing a task folder
does not change the saved result.

There is no approval state. `Ready` means agent completion and project verification passed. It does
not mean that a person approved the UI. Token counts do not measure UI quality.

## Optional Codex assessment

Press `m` in comparison view, or run `compare <task-a> <task-b> --assess`. This starts a separate
local Codex session with a read-only sandbox. It reads the saved code and evidence. It does not
start a browser or perform visual review. It produces a Markdown assessment and its own JSON report,
usage, events, command, environment, and traces. Its token use is separate from task token use.

The app keeps one saved assessment. It binds the exact two run IDs and their order. Use
`compare <task-a> <task-b> --saved` to read it without starting Codex. Starting an assessment
replaces the previous assessment. Rerunning either task removes it. Swapping the pair does not reuse
an assessment written in the other direction. An active assessment holds the execution lock, so a
task cannot replace its evidence while the assessment reads it.

## Files and ownership

```text
apps/agent-ui/src/
  application.rs   Shared task, run, comparison, and preview actions
  project.rs       Task discovery and preflight input checks
  task.rs          Pure task.toml package and permission rules
  task_result.rs   Task result slots and saved selection
  storage.rs       Index, report writes, replacement, locks, and recovery
  runner.rs        Setup, agent execution, and verification
  worker.rs        Worker cancellation and join ownership
  workspace.rs     Saved inputs, copied app, packages, and source inventory
  journal.rs       Measured phases and saved progress
  report.rs        Task result and lifecycle rules
  comparison.rs    Pure differences between saved results
  assessment.rs    Separate Codex assessment and its measurements
  codex/           Private session, command, event decoding, and trace capture
  process.rs       Child groups, cancellation, and output capture
  preview.rs       One preview process and readiness state
  tui/
    state.rs       Task selection, pair, forms, and application actions
    tasks.rs       Task search, stable selection, and list window
    details.rs     Task and comparison content
    input.rs       Keyboard and mouse input
    layout.rs      Shared display and input rectangles
    view.rs        Rendering

~/Library/Application Support/Agent UI/
  active.lock
  current.json             One result slot per task
  selection.json           Selected task and comparison pair
  private/                 Credentials; never public evidence
  assessment/              One optional assessment of an exact run pair
    report.json
    comparison.json
    assessment.md
    prompt.txt
    command.json
    environment.json
    codex-config.toml
    events.jsonl
    stderr.log
    rollout-*.jsonl
  runs/<run-id>/
    report.json
    inputs/                Exact task folder copied at run start
    app/                   Generated app; open this in your editor
    agent-report.md
    evidence/
      prompt.txt
      command.json
      environment.json
      codex-config.toml
      events.jsonl
      codex.stderr.log
      task-config.json
      setup-package.json
      setup-pnpm-lock.yaml
      setup-pnpm-workspace.yaml
      install.log
      verify.log
      rollout-*.jsonl
```

`Application` owns the live workers and a preview per task. CLI and TUI use the same actions.
`Store` owns replacement and recovery. `Comparison` computes differences from values in memory.
`Assessment` owns the optional agent review. See [ARCHITECTURE.md](ARCHITECTURE.md).

The app copies `task.md`, optional `AGENTS.md`, and `references/` to the project root. It preserves
the full task folder in `inputs/`. The bare starter has Astryx packages and no Astryx CLI. The
runner does not add task instructions. There is no import or migration of old run history.

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
when needed. Missing configuration evidence remains `null`.

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

Cancellation stops the owned process group. Quitting stops active work and all owned previews.
Reports remain on disk. On restart, an unfinished report without an active storage lock is marked
`interrupted`. Runs do not resume automatically. Only one run or assessment executes per storage
location. Each preview uses a separate local port and stops when its owning app or CLI command
closes. A new run removes only the previous output for its task and any assessment that uses it.

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
