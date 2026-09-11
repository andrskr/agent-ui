# Agent UI

A local application for coding experiments. The CLI and TUI use the same Rust runner. The terminal
view uses a charcoal background, a violet accent, and separate colours for run states.

Click a run, tab, or review button to select it. In the New experiment form, click a field to focus
it. Click its arrows to change the task or effort. The mouse wheel selects runs in the sidebar and
scrolls the Activity or Report view. Keyboard controls remain available through `?`. The layout
supports terminals of at least 76 columns and 24 rows. It respects `NO_COLOR`.

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

The application needs Cargo, Node, Vite+, Codex, Git, and ripgrep. It currently targets macOS. The
editor action uses Visual Studio Code. Browser actions use the default browser. Run
`vp run agent-ui doctor` to check the local paths. Use `--codex <native-binary>` if the app cannot
resolve your Codex launcher. No task configuration format is supported yet.

## Run flow

1. Select a task folder by ID. Folders that start with `_` are templates.
2. Press Enter or click Run experiment. The app locks run execution in this storage location.
3. Copy the starter and task inputs. Install the fixed lockfile in the copied project.
4. Start Codex with the exact task prompt and a fresh configuration.
5. Save events as they arrive. Show reported usage after each completed turn.
6. Run the generated project's `verify` command. Save its output and exit code.
7. Open the code or start a dev server for human review.

`Ready` means Codex completed and verification passed. It does not mean the UI is approved. There is
no automatic visual judge, score, repair loop, or comparison service.

## Files and ownership

```text
apps/agent-ui/src/
  main.rs        Application entry point
  cli.rs         Command parsing and dispatch
  settings.rs    Typed effort and run settings
  artifact.rs    Output targets shared by the CLI and TUI
  tui/
    mod.rs       Terminal setup, event loop, and restoration
    state.rs     Selection, forms, and run actions
    input.rs     Keyboard and mouse input
    layout.rs    Rectangles shared by rendering and mouse input
    view.rs      Rendering from app state
    theme.rs     Terminal colours
  runner.rs      Worker ownership; setup, agent, and verification phases
  process.rs     Child groups, cancellation, prompt input, and log capture
  codex.rs       Codex arguments, environment, authentication, and traces
  storage.rs     Task lookup, file copies, reports, locks, and removal
  report.rs      Saved report format, defaults, and event reduction
  preview.rs     Editor actions, dev servers, and preview startup state

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
      install.log
      install.stderr.log
      login.log
      login.stderr.log
      verify.log
      verify.stderr.log
      rollout-*.jsonl       Codex session traces, when available
```

The runner owns one worker and the run lock. Joining the worker consumes its handle. Dropping an
active handle requests cancellation and waits for the worker. The process owner stops its child
group before it joins the prompt writer, including when an event callback fails.

The preview owner keeps startup and ready states together. Repeated preview clicks cannot open a
server before it is ready. The CLI and TUI share output lookup and cancellation. Storage and session
paths have read-only accessors. Recovery skips a busy run lock but returns other storage errors.

The saved report remains version 1. This refactor does not require a run-data migration. The app
uses the existing synchronous worker and terminal event loop. It has no extra async runtime or
generic agent-provider layer.

The app copies `task.md`, optional `AGENTS.md`, and `references/` to the project root. It also
preserves the full task folder in `inputs/`. A `task.toml` file produces an explicit error. The
baseline starter has Astryx packages and no Astryx CLI. Package installation is a setup phase. The
runner does not add packages or instructions based on a hidden context profile.

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
usage, completion rules, source changes, ID validation, and process-result classification. Expected
results come from fixed examples. The tests do not create files, start child processes, change the
host environment, or open browsers or editors. There are no temporary project fixtures or shell test
programs.

These tests do not check real file copies, locks, process cleanup, authentication, or browser and
editor integration. Those paths need a separate manual check when requested. The smoke task starts
real Codex and writes run files. It is not part of the test suite. Earlier manual checks are not
proof that these paths still work after a later change.

The `verify:agent-ui` task runs all format, Clippy, and Rust test checks without Vite+ caching.
Cargo and web build commands still write normal build output and tool caches. The no-file rule
applies to test execution, not compilation.

The installed Codex version marks `skip_host_skill_discovery` as under development. The app enables
it to prevent host skill discovery. Its warning remains visible in activity and raw events. Treat
configuration separation as a tested local setup, not a stable guarantee across Codex versions.
