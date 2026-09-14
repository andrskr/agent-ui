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

The Overview shows estimated API cost in USD, one duration breakdown, one token table, verification,
changed file paths, and an excerpt from the saved agent note. File labels mean added (`A`), modified
(`M`), deleted (`D`), or unknown (`?`). The note excerpt uses up to four non-empty lines and 600
characters. Open the full note with `a`. Missing notes and measurements remain explicit.

Preview and Open code stay visible across Overview, Activity, and Evidence. The preview control
shows startup and ready states. Stop applies to the preview owned by this app. Evidence provides the
JSON report, full agent note, and evidence folder through `r`, `a`, and `f`.

Run from the repository root:

```sh
vp run agent-ui
vp run agent-ui providers
vp run agent-ui tasks
vp run agent-ui run smoke
vp run agent-ui show smoke
vp run agent-ui open smoke code
vp run agent-ui preview smoke
vp run agent-ui compare <task-a> <task-b>
vp run agent-ui compare <task-a> <task-b> --assess
vp run agent-ui compare <task-a> <task-b> --saved
```

Use `--provider`, `--model`, `--effort`, and `--timeout` to select a run. The default is Codex,
`gpt-5.6-luna`, `low`, and 300 seconds. Claude defaults to `claude-sonnet-5` with `high` effort. Run
`providers` to print the full catalog without checking tools or login. The model must be available
to your account. Provider errors are saved with the run.

The Run and Assess forms have Provider, Model, and Effort lists. Models show readable names. Use Up
and Down to select a field. Use Left and Right to select a value. Tab and Shift+Tab also move
between fields. Click a field to select it, or click its arrows to change the value. A provider
change resets the model and effort. A model change resets an incompatible effort. `default` lets the
provider CLI choose its effort. Custom model IDs are available through `--model` on the CLI. A
binary override applies only to the selected provider.

### Providers and model choices

The catalog was checked on 2026-09-14. It is a local list, not an account entitlement check.

| Provider | Model IDs                                                                                                      | Efforts                               |
| -------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| Codex    | `gpt-6-astra`, `gpt-5.6-sol`, `gpt-5.6-terra`                                                                  | low, medium, high, xhigh, max, ultra  |
| Codex    | `gpt-5.6-luna`                                                                                                 | low, medium, high, xhigh, max         |
| Codex    | `gpt-5.5`, `gpt-5.3-codex-spark`                                                                               | low, medium, high, xhigh              |
| Claude   | `claude-sonnet-5`, `claude-opus-5`, `claude-fable-5-1`, `claude-fable-5`, `claude-opus-4-8`, `claude-opus-4-7` | low, medium, high, xhigh, max         |
| Claude   | `claude-sonnet-4-6`, `claude-opus-4-6`                                                                         | low, medium, high, max                |
| Claude   | `claude-haiku-4-5-20251001`, `claude-opus-4-5-20251101`, `claude-sonnet-4-5-20250929`                          | Not supported; no effort flag is sent |

Claude also accepts `sonnet`, `opus`, `fable`, and `haiku` aliases. The picker has one entry per
model, with aliases mapped to that entry. Use full IDs for comparisons with fixed model choices.
Custom model IDs remain available through the CLI.

Each provider owns a `catalog.rs` file. The bundled catalog includes all six visible models in the
installed Codex catalog and all eleven active general-purpose Claude models checked on 2026-09-14.
Codex Spark remains selectable even though the Codex catalog marks it as unavailable through the
API. The app uses the Codex CLI with subscription login. It does not filter this list by API access.

The identifiers were checked against
[CodexBar's model prices at a5f2c58](https://github.com/steipete/CodexBar/blob/a5f2c581ce2e859dab983e28af50c03351db7dd3/Sources/CodexBarCore/Vendored/CostUsage/CostUsagePricing.swift).
That file includes historical and API-only models. A price entry alone does not make a model
selectable. Codex choices and efforts come from its installed model catalog. Claude choices also use
[current model status](https://platform.claude.com/docs/en/about-claude/model-deprecations) and
[Claude Code effort support](https://code.claude.com/docs/en/model-config#adjust-effort-level).
Retired Claude models stay in historical pricing code but are absent from the picker.

The catalog is bundled for use without provider tools or login. It does not query the account or
refresh itself. Update only the provider's `catalog.rs` when supported models or efforts change.
Account access and managed policy can further limit these choices.

```sh
vp run agent-ui --provider claude login
vp run agent-ui --provider claude doctor
vp run agent-ui --provider claude --model claude-fable-5-1 --effort high run smoke
vp run agent-ui --provider codex --model gpt-6-astra --effort ultra run smoke
vp run agent-ui --provider claude compare <task-a> <task-b> --assess
```

Use `--project <folder>` when you start outside this repository. Use `--data-dir <folder>` to choose
a different storage location. It must be outside the repository.

Building the app needs Cargo. Opening the built UI and inspecting saved runs does not need a
provider CLI, Node, Vite+, Git, or ripgrep. A run resolves these tools during setup, in the
background. It uses the ripgrep bundled with Codex when available, then checks PATH. Preview needs
Vite+ only. The app currently targets macOS. The editor action uses Visual Studio Code. Browser
actions use the default browser. Run `vp run agent-ui doctor` to check the local paths. Use
`--binary <native-binary>` if the app cannot resolve the selected provider launcher. Optional
`task.toml` files set task packages.

## Tasks and comparison

The sidebar lists tasks. Each task keeps one run. Select a task to see its latest result or its
prompt if it has no result. Press `n` to start it. The Run form lets you select provider, model, and
effort.

Starting a run removes that task's previous code, logs, and reports. The app validates settings,
then starts the run in the background. The run stops that task's owned preview, removes any
assessment that uses the old run, and deletes the old output before it creates the new run. It then
checks tools and provider sign-in. A failed or cancelled new run remains the current result, and a
sign-in or tool failure is saved as a failed run. There is no rollback or run history.

If removal fails, the app shows `Cleanup blocked`. No new run starts. Restart the app or run that
task again to retry cleanup. A preview owned by another process blocks removal. Stop that preview
first. The app does not stop previews for other tasks.

Press `c` to select a second task. Both tasks must have a saved run. Tasks with no run remain
visible in the picker, with a reason why they cannot be selected. The selected task is side A. The
other task is side B. Use `s` to swap them, `v` to choose a side, and `Esc` to return to the task
view. The app saves the selected task and comparison pair.

Overview shows estimated API cost, time, and token differences as **B minus A**. It also shows
provider, model, effort, tool versions, verification, and saved input, setup, and final source
changes. Activity and Evidence show the selected side. Code and preview actions also use the
selected side. Both previews can remain open at the same time. The comparison reads saved input
hashes and reports. Editing a task folder does not change the saved result.

There is no approval state. `Ready` means agent completion and project verification passed. It does
not mean that a person approved the UI. Token counts do not measure UI quality.

## Optional agent assessment

Press `m` in comparison view, or run `compare <task-a> <task-b> --assess`. The form lets you select
its provider, model, and effort. It starts a separate local session with the provider's assessment
permissions. It reads the saved code and evidence. It does not start a browser or perform visual
review. It produces a Markdown assessment and its own JSON report, usage, events, command,
environment, and provider artifacts. Its token use is separate from task token use.

The app keeps one saved assessment. It binds the exact two run IDs and their order. Use
`compare <task-a> <task-b> --saved` to read it without starting an agent. Starting an assessment
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
  cost.rs          Shared cost evidence and USD formatting
  assessment.rs    Separate agent assessment and its measurements
  providers/       Registry, catalog, shared execution contract
    codex/         Codex session, events, and CodexBar pricing
    claude/        Claude session, events, and CodexBar pricing
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
    <provider settings file>
    events.jsonl
    agent.stderr.log
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
      <provider settings file>
      events.jsonl
      agent.stderr.log
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
The report also records wall-clock timestamps, binary versions, provider, requested model and
effort, provider session ID, exit codes, input hashes, source changes, and event counts.

Input tokens include cache reads and cache writes. Reasoning tokens, when supplied, are part of
output tokens. Missing usage is `null`, not zero. The requested model is not a claim about the model
actually served. Raw events and available rollouts are kept for further analysis. Their internal
format can change across provider CLI versions.

### Estimated API cost

The Overview and comparison show an approximate cost in USD. This is token use priced at API rates.
It is not a subscription charge. The JSON report stores `cost_usd`, `cost_note`, `cost_basis`,
`cost_models`, and `cost_source`. Comparison JSON includes `measurements.estimated_cost_usd`.
Assessment usage remains separate from task costs.

For Codex, the app uses the bundled rates and calculation from
[CodexBar at commit a5f2c58](https://github.com/steipete/CodexBar/blob/a5f2c581ce2e859dab983e28af50c03351db7dd3/Sources/CodexBarCore/Vendored/CostUsage/CostUsagePricing.swift).
The rates were copied on 2026-09-14. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). The app
pins these rates for repeatable comparisons. It does not read CodexBar settings, custom prices, or
the optional models.dev cache. Rate updates require a code change.

For a run with complete saved request records, the app adds each request cost:

```text
cost = new input × input rate
     + cached input × cache-read rate
     + cache writes × cache-write rate
     + output × output rate
```

Input includes cache reads and writes. The app subtracts these subsets before it prices new input.
It does not add reasoning tokens again. It uses each recorded model, timestamp, and service tier. It
applies CodexBar's long-context and API Fast rules to each request. It also uses CodexBar's
historical Luna and Terra rates before 2026-07-30. Repeated token totals do not add cost.

The request totals must match the saved run usage when that usage is available. Missing records,
gaps, resets, and unreadable traces use a marked fallback. This fallback uses the requested model
and standard rates for the run totals. It cannot apply request-specific long-context, Fast, or
cache-write prices. A large run total alone does not trigger long-context pricing.

Unknown model prices remain unavailable. Provider prefixes cannot select another provider's price.
CodexBar assigns a zero rate to the Spark research preview; the view states this. Positive costs
below one cent show `<$0.01`. Calculations and differences retain full precision in JSON.

New reports save their cost and source. The report format is schema 2. Old reports are not
supported. There is no migration, cost backfill, or old-report cache.

For Claude, the final `total_cost_usd` from the CLI takes priority. It includes the CLI's model
pricing. See [Claude cost tracking](https://platform.claude.com/docs/en/agent-sdk/cost-tracking).
The adapter also borrows CodexBar's message replacement and cache rules from its
[Claude scanner](https://github.com/steipete/CodexBar/blob/a5f2c581ce2e859dab983e28af50c03351db7dd3/Sources/CodexBarCore/Vendored/CostUsage/CostUsageScanner%2BClaude.swift).
It replaces repeated records with the latest usage for the same message and request. If request IDs
are absent, it uses the stable message ID. Missing message IDs make this fallback unavailable. Final
usage replaces observed message totals. These two sources are never added together.

Claude's fallback uses the same pinned CodexBar price table, five-minute and one-hour cache-write
rates, and per-request context thresholds. The table does not yet price Sonnet 5, Opus 5, or Fable
5.1. Those models need the CLI's reported cost. The app does not guess their prices. A fallback
requires complete observed usage and known prices for every message. When a final result exists,
observed usage must also agree with final usage. Interrupted streams can show partial estimates.

Cancellation stops the owned process group. Quitting stops active work and all owned previews.
Reports remain on disk. On restart, an unfinished report without an active storage lock is marked
`interrupted`. Runs do not resume automatically. Up to four task runs execute at once, one run per
task. An assessment does not run while any task run is active. One app instance uses a storage
location at a time. Each preview uses a separate local port and stops when its owning app or CLI
command closes. A new run removes only the previous output for its task and any assessment that uses
it.

Codex gets a fresh HOME and CODEX_HOME, an explicit environment, disabled external integrations, and
the workspace-write sandbox. The app copies the existing file-based ChatGPT login into private
storage on the first run. It does not copy global Codex configuration or write back to global auth.
If no usable login exists, run `vp run agent-ui login` to sign in with a separate device flow. Only
the private credential seed keeps refreshes. Per-run private state is removed after trace capture.

Claude needs the native Claude Code CLI, version 2.1.257 or later. Run the Claude login command
above once for each data directory. The adapter sets `CLAUDE_CONFIG_DIR` to
`private/providers/claude`. Claude Code owns login and credential refresh. Agent UI does not copy or
read the host keychain. Each operation has a fresh HOME. Safe mode disables user customizations,
hooks, skills, plugins, MCP, and automatic memory. Managed machine policy can still apply.

Claude task runs expose Read, Glob, Grep, Edit, Write, and sandboxed Bash. They use restricted mode
and `acceptEdits`. Unsandboxed Bash fallback is disabled. Assessments expose only Read, Glob, and
Grep, with the two run folders added for reading. A denied required tool makes the operation fail.
Task `AGENTS.md` is passed explicitly as additional instructions. Assessment task files remain
reference evidence. Automatic model switching on content flags is disabled.

This is configuration separation, not full host isolation. Both providers use the same macOS
account. Host reads, installed binaries, system policy, account limits, and provider caching can be
shared. Dependency setup, verification, and preview execute project code on the host. Use trusted
tasks. Raw logs and traces can contain task content and paths. Review them before sharing them.

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
memory. They do not install packages or run an agent. Expected results come from fixed examples. The
tests do not create files, start child processes, change the host environment, or open browsers or
editors. There are no temporary project fixtures or shell test programs.

These tests do not check real file copies, locks, process cleanup, authentication, or browser and
editor integration. Those paths need a separate manual check when requested. The smoke task starts a
real provider and writes run files. It is not part of the test suite. Earlier manual checks are not
proof that these paths still work after a later change.

The TUI uses Chrono for local date conversion. Ratatui remains pinned to 0.30.2. Its
`unstable-rendered-line-info` feature supplies the wrapped line count used for scrolling and mouse
targets. Review this feature when updating Ratatui.

The `verify:agent-ui` task runs all format, Clippy, and Rust test checks without Vite+ caching.
Cargo and web build commands still write normal build output and tool caches. The no-file rule
applies to test execution, not compilation.

The installed Codex version marks `skip_host_skill_discovery` as under development. The app enables
it to prevent host skill discovery. Its warning remains visible in activity and raw events.
Configuration separation needs a live check after provider CLI updates.

### Live checks on the run laptop

The provider refactor has no live execution proof yet. Use a separate output folder for these
checks.

1. Run `providers` and each provider's `doctor`. Check the installed CLI versions.
2. Run each provider's `login`. Use a model that the account can access.
3. Run a small task through each provider. Check the requested model, effort, usage, and USD cost.
4. Check that Claude receives the task's `AGENTS.md` and that required sandboxed tools can run.
5. Compare the two tasks. Run an assessment through each provider. Confirm that source files do not
   change.
6. Cancel a run. Check partial evidence and that the owned process group stops.
7. Start a replacement with invalid settings or missing login. Confirm that old output remains.

See [the provider boundary](ARCHITECTURE.md#provider-boundary) for the extension steps.
