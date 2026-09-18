# Agent UI

A local application for coding experiments. The CLI and TUI use the same Rust runner. The terminal
view uses a charcoal background, a mint accent, and separate colours for run states.

The sidebar shows groups with their tasks below them. For example, Smoke contains Baseline, Context,
and Repair. The tree stays visible at both navigation levels. Up and Down select groups. Press Enter
to move into a group. Up and Down then select only its tasks. Press Esc to return to the group.
Click a group or task to select it directly. The mouse wheel follows the current level. Press `/` to
search by group, task, or saved state. Search words must all match.

The sidebar shows each group's task count, status counts, and latest run start date. Counts include
all group members, even during search. Task rows show the current state and run start date. Active
tasks show their phase and elapsed time. `Prev` marks an older result while a replacement is queued,
starting, or failed to start. Dates use local time. Tasks with no run have no date.

Group Overview shows all tasks in the selected group, even when search hides some sidebar rows.
Press `n`, or click Run all tasks, to choose one provider, model, and effort for the group. The form
states that each new run replaces its old output. Tasks already running or queued are skipped. The
app runs up to four tasks at once. It queues the rest and starts them as slots become free. The
queue stays in memory. Quitting removes pending work and stops active work. Press `C` at group level
to cancel that group's active and queued tasks. A failure in one task does not stop the rest.

Every view uses the same tab controls. Left and Right, Tab and Shift+Tab, or the visible tab labels
change views. Number keys select the visible tabs. Group tabs are Overview and Compare. Task tabs
are Overview, Activity, and Setup. Tab changes the view. Enter and Esc change the navigation level.
Use `?` for all keyboard controls. The layout needs at least 76 columns and 24 rows. It respects
`NO_COLOR`.

Compare opens the saved pair for this group, or the first two available results. If fewer than two
results exist, it shows an empty state. Use `a` or `b` to choose either member. Use `s` to swap
them. Both selectors stay inside the current group. Compare shows configurations, outcomes, and
metrics with **B minus A** differences. It has no run, preview, file, or assessment actions. Esc
returns to Group Overview. Enter opens Task View. Up and Down select groups and keep Compare open.

In Task View, use `n` to run the selected task and `c` to compare it with another variant in the
same group. This opens the group Compare view. Task Overview shows estimated API cost in USD, phase
durations, tokens, verification, and a short agent note. Activity shows the event log. Setup shows
saved run configuration and the current task prompt. It marks changed task inputs. Missing
measurements stay explicit. Generated file inventories and raw JSON are not shown.

Each view has one local action row. Task Run, Preview, and Code actions keep the same position
across task tabs. The preview control shows startup and ready states. Stop applies to the preview
owned by this app. Use `r`, `a`, and `f` to open the report, full agent note, and run files.

See [the view design](../../docs/ui-design/README.md) for the design board and interaction rules.

Run from the repository root:

```sh
vp run agent-ui
vp run agent-ui providers
vp run agent-ui tasks
vp run agent-ui tasks --check
vp run agent-ui run invite-member--baseline
vp run agent-ui show invite-member--baseline
vp run agent-ui open invite-member--baseline code
vp run agent-ui preview invite-member--baseline
vp run agent-ui compare invite-member--baseline invite-member--context
vp run agent-ui compare invite-member--baseline invite-member--context --assess
vp run agent-ui compare invite-member--baseline invite-member--context --saved
```

Use `--provider`, `--model`, `--effort`, and `--timeout` to select a run. The default is Codex,
`gpt-5.6-luna`, `low`, and 900 seconds. Claude defaults to `claude-sonnet-5` with `high` effort. Run
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
vp run agent-ui --provider claude --model claude-fable-5-1 --effort high run invite-member--baseline
vp run agent-ui --provider codex --model gpt-6-astra --effort ultra run invite-member--baseline
vp run agent-ui --provider claude compare invite-member--baseline invite-member--context --assess
```

Use `--project <folder>` when you start outside this repository. Use `--data-dir <folder>` to choose
a different storage location. It must be outside the repository.

Building the app needs Cargo. Opening the built UI and inspecting saved runs does not need a
provider CLI, Node, Vite+, Git, pnpm, or ripgrep. A run resolves these tools during setup, in the
background. It uses the ripgrep bundled with Codex when available, then checks PATH. It resolves
pnpm through Vite+ so a different global version cannot trigger a download inside the agent sandbox.
The agent's `PATH` holds Node, ripgrep, and a wrapper for pnpm and for the `vp` command of the
copied project, then the system folders. A wrapper keeps the real path of the target, because the
pnpm that Vite+ manages reads its own files beside that path. The agent can therefore run the
project's `package.json` scripts by name. Each task prompt also gets notes that name these commands.
Preview needs Vite+ only. The app currently targets macOS. The editor action uses Visual Studio
Code. Browser actions use the default browser. Run `vp run agent-ui doctor` to check the local
paths. Use `--binary <native-binary>` if the app cannot resolve the selected provider launcher.
Optional `task.toml` files set task packages.

## Batch execution and permanent results

`--record` saves permanent metrics and run activity. Logs survive task replacement. Use
`ledger events` to read an old run without opening its generated workspace. See the
[activity guide](../../docs/cli-run-activity.md) for SQL examples and capture limits.

Run a named suite with one configuration:

```sh
vp run agent-ui batch run --suite ui-evaluation --provider claude --model claude-sonnet-5 --dry-run
vp run agent-ui batch run --suite ui-evaluation --provider claude --model claude-sonnet-5 --record
vp run agent-ui batch show <batch-id>
vp run agent-ui batch resume <batch-id>
vp run agent-ui batch resume <batch-id> --retry-incomplete
vp run agent-ui ledger info
vp run agent-ui ledger events <run-id> --json
```

The initial `ui-evaluation` suite selects the three Invite Member variants. Add explicit scenarios
to `experiments/suites/ui-evaluation.toml` as they become available. Missing tasks fail validation.
The dry run prints the exact selection and configuration without writing files or running agents.

`--record` saves every result in SQLite, including unsuccessful attempts. Without this option, the
batch is temporary and cannot resume. Each task still keeps only its latest generated artifacts. The
SQLite ledger retains earlier metrics after those artifacts are removed. Recording needs no manual
inspection or approval. Open the TUI after the batch if you want to inspect its latest code.

The command prints its batch ID and database path. Progress goes to stderr; the final execution
summary is JSON on stdout. Exit codes are 0 for success, 1 for incomplete execution or storage
errors, 2 for invalid plans, and 130 for Ctrl+C.

Resume uses saved inputs and configuration. It starts pending tasks only. `--retry-incomplete` also
creates new attempts for unsuccessful tasks. It never repeats successful tasks. Resume rejects
provider, model, effort, timeout, and binary overrides. Use a new batch to change configuration.

The database schema, metric units, and read-only SQL examples are in
[the ledger guide](../../docs/cli-ledger.md). Comparison, exports, and report generation are
separate work; these commands only execute tasks and record evidence.

## Tasks and comparison

Name each task folder `<group>--<variant>`. For example:

```text
experiments/tasks/
  _template/
  invite-member--baseline/task.md
  invite-member--baseline/AGENTS.md
  invite-member--context/task.md
  invite-member--context/AGENTS.md
  invite-member--repair/task.md
  invite-member--repair/AGENTS.md
```

Each part uses lowercase ASCII letters, numbers, and single hyphens between words. Use exactly one
`--` separator. Neither part can be empty or start or end with a hyphen. The full ID is limited to
120 characters. `_template` is excluded. Other task names are invalid. The folder name is the only
source of group membership. See [the experiment instructions](../../experiments/AGENTS.md).

A group identifies the work and success criteria. A variant identifies its setup. Each variant owns
its prompt and optional instructions, references, and package settings. Copy an existing variant to
add another setup. Keep the main objective and success criteria the same. The app checks group
names; it cannot prove that two prompts request equivalent work. Input and setup differences remain
visible in the comparison.

All 12 scenarios have matching prompts across their three variants. Every variant has the same
general workflow in `AGENTS.md` and receives the shared starter's Astryx and Recharts runtime
packages. `baseline` has only the general instructions. `context` adds the Astryx CLI and
Astryx-specific guidance. `repair` adds the root-quality setup profile and requires
`vp lint src --fix`, `vp fmt src --write`, and `vp run repair` within the same run. This workflow
applies to old and new tasks. Every variant starts from the starter. A variant named `repair` does
not inherit another run's output or start a separate agent pass.

Run `vp run agent-ui tasks --check` after changing task inputs. It checks names, prompts, input
files, and package settings without opening run storage or starting an agent.

Each task keeps one run. Enter a group and select a task to see its latest result or its prompt if
it has no result. Press `n` to start it. The Run form lets you select provider, model, and effort.
The selected task, navigation level, and comparison pair are saved.

Starting a run removes that task's previous code, logs, and reports. The app validates settings,
then starts the run in the background. The run stops that task's owned preview, removes any
assessment that uses the old run, and deletes the old output before it creates the new run. It then
checks tools and provider sign-in. A failed or cancelled new run remains the current result, and a
sign-in or tool failure is saved as a failed run. There is no rollback or artifact history. Recorded
CLI batches retain historical metrics separately in SQLite.

If removal fails, the app shows `Cleanup blocked`. No new run starts. Restart the app or run that
task again to retry cleanup. A preview owned by another process blocks removal. Stop that preview
first. The app does not stop previews for other tasks.

Press `c` to select another variant from the same group. Both variants must have a saved run. The
picker excludes the selected task and all other groups. Variants with no run remain visible with a
reason why they cannot be selected. The app explains when the group has no other variant. The
selected task is A. The other task is B. Use `s` to swap them and `Esc` to return to Group Overview.
The app saves the selected task and comparison pair. On startup, it clears an unavailable or invalid
pair and returns to Overview with a message. CLI comparison, assessment, and saved assessment access
use the same group rule.

Compare View shows estimated API cost, phase times, and token differences as **B minus A**. It also
shows provider, model, effort, time limit, package settings, and verification outcomes. It reads
saved reports. Editing a task folder does not change the saved result. Tool versions remain in Task
Setup. The CLI comparison report still includes file differences for detailed inspection.

There is no approval state. `Ready` means agent completion and project verification passed. It does
not mean that a person approved the UI. Token counts do not measure UI quality.

## Optional agent assessment

Run `compare invite-member--baseline invite-member--context --assess` from the CLI. Set its
provider, model, and effort through CLI options. This starts a separate local session with the
provider's assessment permissions. It reads the saved code and evidence. It does not start a browser
or perform visual review. It produces a Markdown assessment and its own JSON report, usage, events,
command, environment, and provider artifacts. Its token use is separate from task token use.

The app keeps one saved assessment. It binds the exact two run IDs and their order. Use
`compare invite-member--baseline invite-member--context --saved` to read it without starting an
agent. Starting an assessment replaces the previous assessment. Rerunning either task removes it.
Swapping the pair does not reuse an assessment written in the other direction. An active assessment
holds the execution lock, so a task cannot replace its evidence while the assessment reads it.

## Files and ownership

```text
apps/agent-ui/src/
  application.rs   Shared task, run, comparison, and preview actions
  project.rs       Task discovery and preflight input checks
  task.rs          Task identity, comparison groups, and task.toml rules
  task_result.rs   Task result slots and saved selection
  storage.rs       Index, report writes, replacement, locks, and recovery
  runner.rs        Setup, agent execution, and verification
  worker.rs        Worker cancellation and join ownership
  run_queue.rs     Pending group tasks and dispatch capacity
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
    tasks.rs       Group and task navigation, search, and list window
    groups.rs      Group Overview and per-task summaries
    tabs.rs        Shared tab rendering and keyboard/mouse targets
    compare.rs     Comparison layout, content, and controls
    text.rs        Shared text formatting
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
experiments/tasks/workspace-settings--baseline/
  task.md
  task.toml       Optional package settings
  AGENTS.md       Optional project instructions
  references/    Optional input assets
```

For a task that needs the Astryx CLI, use:

```toml
[dev-dependencies]
"@astryxdesign/cli" = "0.6.2"
"gpt-tokenizer" = "3.4.0"

[allow-builds]
"@astryxdesign/cli@0.6.2" = true
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

For Codex, the app follows
[CodexBar at commit 639b155](https://github.com/steipete/CodexBar/blob/639b15522692ead0e9a26f78cebd56a0803ffc8b/Sources/CodexBarCore/Vendored/CostUsage/CostUsagePricing.swift).
The prices include a models.dev snapshot from 2026-09-16. CodexBar prefers this catalog over its
built-in prices, except for its explicit historical rules. The app pins the resolved prices for
repeatable comparisons. It does not read CodexBar settings or download prices during a run. See
[the cost review](docs/cost-estimation.md), [the saved prices](docs/cost-prices-2026-09-16.json),
and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

For complete saved request records, the app adds each request cost:

```text
cost = new input × input rate
     + cached input × cache-read rate
     + cache writes × cache-write rate
     + output × output rate
```

Input includes cache reads and writes. The app subtracts these subsets before it prices new input.
It does not add reasoning tokens again. Each request uses its recorded model, timestamp, and service
tier. Long-context and API Fast rules apply per request. Repeated totals do not add cost. When both
cache-read field names occur, the larger value is used. The two values are not added.

Request totals must match saved run usage when available, including reported cache writes. Missing
records, gaps, resets, and unreadable traces use a marked fallback. It prices run totals with the
requested model, standard rates, and reported cache writes. If a model has a context price threshold
and total input exceeds it, the fallback is unavailable. It cannot prove individual request sizes.
The fallback does not infer Fast mode or the actual served model.

Unknown model prices remain unavailable. Provider prefixes cannot select another provider's price.
Spark now uses the nonzero catalog price. Zero usage and missing prices remain different. Positive
costs below one cent show `<$0.01`. JSON stores full calculation precision. New reports save their
cost and source. Saved reports are not repriced. There is no cost backfill.

For Claude, the final `total_cost_usd` from the CLI takes priority. It includes the CLI's model
pricing. See [Claude cost tracking](https://platform.claude.com/docs/en/agent-sdk/cost-tracking).
The message fallback uses CodexBar's repeated-message and cache rules. It replaces repeated records
with the latest usage for the same message and request. If request IDs are absent, it uses the
stable message ID. Missing message IDs make this fallback unavailable. Final usage replaces observed
message totals. These two sources are never added together.

Claude's fallback includes five-minute and one-hour cache-write prices, model-specific cache-read
prices, and per-request context thresholds. The pinned catalog includes Sonnet 5, Opus 5, and Fable
5.1. A fallback requires complete observed usage and known prices for every message. When a final
result exists, observed usage must also agree with final usage. Interrupted streams can show partial
estimates.

Cancellation stops the owned process group. Quitting stops active work and all owned previews.
Reports remain on disk. On restart, an unfinished report without an active storage lock is marked
`interrupted`. Runs do not resume automatically. Up to four task runs execute at once, one run per
task. An assessment does not run while any task run is active or queued. One app instance uses a
storage location at a time. Each preview uses a separate local port and stops when its owning app or
CLI command closes. A new run removes only the previous output for its task and any assessment that
uses it.

Codex gets a fresh HOME and CODEX_HOME, an explicit environment, disabled external integrations, and
the workspace-write sandbox. The app copies the existing file-based ChatGPT login into private
storage on the first run. It does not copy global Codex configuration or write back to global auth.
If no usable login exists, run `vp run agent-ui login` to sign in with a separate device flow. Only
the private credential seed keeps refreshes. Per-run private state is removed after trace capture.

Claude needs the native Claude Code CLI, version 2.1.257 or later. Run the Claude login command
above once for each data directory. The adapter sets `CLAUDE_CONFIG_DIR` to
`private/providers/claude`. Claude Code owns login and credential refresh. Agent UI does not copy or
read the host keychain. Login, sign-in checks, tasks, and assessments keep the same macOS HOME,
USER, and LOGNAME so Claude can read that login. A temporary HOME or a different USER can make
Claude report that it is not signed in. The rest of the run environment uses an explicit list of
values. It does not inherit API keys or authentication overrides. Safe mode disables user
customizations, hooks, skills, plugins, MCP, and automatic memory. Managed machine policy can still
apply.

Claude task runs expose Read, Glob, Grep, Edit, Write, and sandboxed Bash. They use restricted mode
and `acceptEdits`. Unsandboxed Bash fallback is disabled. Assessments expose only Read, Glob, and
Grep, with the two run folders added for reading. The sandbox refuses a command that reaches outside
the workspace. The agent then sends a different command, so a refusal becomes a warning and a count
in activity. It does not fail the run. Task `AGENTS.md` is passed explicitly as additional
instructions. Assessment task files remain reference evidence. Automatic model switching on content
flags is disabled.

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
vp exec pnpm --dir experiments/starter run verify
```

The Rust tests run in memory. They check log records split across reads, token totals, missing
usage, completion rules, source changes, ID validation, and process-result classification. UI tests
check search, stable selection, small viewports, scroll extent, preview states, and mouse targets.
They render into Ratatui memory buffers. Group tests check navigation boundaries, search, mouse
rows, small windows, comparison scope, and queued or failed summaries. Queue tests check capacity,
settings, and cancellation in memory. Task tests parse TOML and apply package settings to JSON in
memory. They do not install packages or run an agent. Expected results come from fixed examples. The
tests do not create files, start child processes, change the host environment, or open browsers or
editors. There are no temporary project fixtures or shell test programs.

These tests do not check real file copies, locks, process cleanup, authentication, or browser and
editor integration. Those paths need a separate manual check when requested. A task run starts a
real provider and writes run files. It is not part of the test suite. Earlier manual checks are not
proof that these paths still work after a later change.

The TUI uses Chrono for local date conversion. Ratatui remains pinned to 0.30.2. Its
`unstable-rendered-line-info` feature supplies the wrapped line count used for scrolling and mouse
targets. Review this feature when updating Ratatui.

The `verify:agent-ui` task runs format, Clippy, Rust tests, and `tasks --check` without Vite+
caching. The task catalog check reads repository inputs. It does not start a task run. Cargo and web
build commands still write normal build output and tool caches. The no-file rule applies to test
execution, not compilation.

The installed Codex version marks `skip_host_skill_discovery` as under development. The app enables
it to prevent host skill discovery. Its warning remains visible in activity and raw events.
Configuration separation needs a live check after provider CLI updates.

### Live checks on the run laptop

On 2026-09-14, Claude Code 2.1.270 completed the smoke task with Sonnet 5 and high effort. A
separate live task used Read, Glob, Grep, Edit, Write, and Bash. It read task instructions, ran Node
and ripgrep, and passed the generated app verification. Claude also completed a comparison
assessment without tool denials. Codex 0.154.0 completed the smoke task with its default model.
These checks used the local account. They do not prove access to every listed model.

Use a separate output folder for further checks.

1. Run `providers` and each provider's `doctor`. Check the installed CLI versions.
2. Run each provider's `login`. Use a model that the account can access.
3. Run a small task through each provider. Check the requested model, effort, usage, and USD cost.
4. Check that Claude receives the task's `AGENTS.md` and that required sandboxed tools can run.
5. Compare two variants from the same group. Run an assessment through each provider. Confirm that
   source files do not change.
6. Cancel a run. Check partial evidence and that the owned process group stops.
7. Start a replacement with invalid settings or missing login. Confirm that old output remains.

See [the provider boundary](ARCHITECTURE.md#provider-boundary) for the extension steps.

### Group UI check

On 2026-09-16, a separate manual terminal check used a temporary project and copied smoke reports.
Enter, Esc, arrow keys, group and task mouse selection, pair selection, and view switching worked. A
six-task group used a missing provider binary to exercise queue dispatch and failed-run handling.
All six tasks reached a failed result, including tasks that waited for a free slot. The copied smoke
results remained ready. This check did not run a provider or establish successful group execution
with a live model.
