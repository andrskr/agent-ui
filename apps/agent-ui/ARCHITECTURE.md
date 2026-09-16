# Architecture

The app runs task experiments and lets a person compare their code and evidence. CLI and TUI use the
same application service. Each task has at most one current run. Comparison joins two different
variants from the same task group. There is no run history, approval state, or promotion step.

## Owners

| Owner               | Responsibility                                                       |
| ------------------- | -------------------------------------------------------------------- |
| `Project`           | Find tasks and validate source inputs before replacement.            |
| `TaskId`            | Parse task names and enforce comparison groups in memory.            |
| `TaskConfig`        | Parse task packages, setup profiles, and the repair requirement.     |
| `SetupPlan`         | Resolve profile files, package conflicts, and named checks.          |
| Repair controller   | Check source snapshots and record required in-pass check evidence.   |
| `Application`       | Coordinate runs, assessments, and previews for both interfaces.      |
| `Store`             | Own the task index, run files, locks, replacement, and recovery.     |
| `Index`             | Enforce current and removing slots in memory.                        |
| `Runner`            | Coordinate setup, agent execution, and verification.                 |
| `PreparedWorkspace` | Copy inputs and starter, install packages, and record source hashes. |
| `Journal`           | Measure phases and save progress and command output.                 |
| `Report`            | Apply task lifecycle and completion rules to typed observations.     |
| `Comparison`        | Compute signed differences from two saved results.                   |
| `Cost`              | Define cost evidence and format USD values.                          |
| `Assessment`        | Run an explicit agent comparison and record its separate evidence.   |
| `RunQueue`          | Keep pending group tasks and settings in memory.                     |
| `Worker<T>`         | Own cancellation and join a background thread.                       |
| `Preview`           | Own one server, port, lock, and readiness state.                     |
| `Process`           | Own child groups and capture output.                                 |
| `Provider`          | Own models, efforts, binaries, login, sessions, and pricing.         |
| `Session`           | Build one command, decode events, and save provider artifacts.       |
| TUI                 | Own selection, forms, search, scroll, and rendering.                 |

The TUI does not read evidence files or own child processes. Pure comparison and rendering do not
start an agent. `toolchain.rs` resolves shared tools and asks the provider to resolve its
executable; it does not select tasks or start previews.

## Provider boundary

`providers/mod.rs` holds the provider registry, catalog types, and execution contract. Shared code
looks up a provider by its string ID. It does not match on Codex or Claude. The model catalog holds
model IDs, aliases, default effort, and valid efforts for each model. Custom IDs use the provider's
custom effort list. `default` omits an explicit effort and lets the CLI choose.

`Settings` validates choices and changes selection through this catalog. A provider change selects
its default model and effort and clears the binary override. A model change resets an incompatible
effort. The CLI and both TUI forms use these same rules.

Each provider owns binary resolution, login checks, credentials, command arguments, event decoding,
artifact capture, and cost calculation. `LocalSession` supplies a temporary HOME, explicit common
environment, and local tool wrappers. Provider code adds its own environment. Claude keeps the host
HOME and user identity so its macOS Keychain login matches the sign-in check. Its configuration
folder remains private to the selected data directory. The shared executor saves raw events and
command evidence, runs the process, and attempts artifact capture on failure. `Report` and
`Assessment` accept only typed observations. Neither reads provider JSON.

To add a provider:

1. Add `providers/<id>/mod.rs`. Implement `Provider` and its `Session`. Put its catalog in that file
   or a separate `catalog.rs`.
2. Add `event.rs` for its JSON protocol. Add `cost.rs` only if it needs local pricing.
3. Declare the module and add one entry in the registry in `providers/mod.rs`.

Shared CLI, forms, runner, assessment, and comparison need no provider-specific branches. Add
in-memory tests beside the adapter. Run the separate live checks for its process and login behavior.
A provider with a new transport can use the same typed evidence, but this contract currently runs
local agent CLIs. Direct HTTP APIs are not implemented.

## Replacement

1. Validate settings, task source, package settings, starter manifest, and executables.
2. Take the storage execution lock. Check provider version and login before deleting output.
3. Stop this application's preview for the selected task.
4. Take the old run's preview lock. A preview in another process blocks replacement.
5. Save the task slot as `Removing(old-id)` before deleting files.
6. Delete an assessment that uses the old run. Delete the old public and private run files.
7. Remove the task slot. Create the new report and register `Current(new-id)`.
8. Start the worker. It keeps the execution lock until all work ends.

This order keeps old output when preflight fails. After step 5, failure leaves a cleanup record. No
replacement starts until cleanup succeeds. Recovery retries removal. A failed or cancelled new run
is the current result. There is no rollback.

Writes use a temporary file, file sync, rename, and parent-directory sync. Recovery holds the same
execution lock. It marks abandoned active runs as `Interrupted`. Unregistered run directories are
incomplete starts or obsolete output. Recovery removes them when their preview lock is free. It does
not import old history. A missing task index starts empty.

## Run lifecycle

```text
Preparing -> Running -> Verifying -> Ready
     |          |           |
     +----------+-----------+-----> Failed / Cancelled / Interrupted
```

Setup preserves the full input folder and loads that saved snapshot. It applies task packages only
to the copied app. It saves package and lock files before agent execution. Setup changes do not
count as agent edits. The journal preserves measurements on errors. The runner attempts trace
capture and final source inventory even if the agent fails.

`Ready` needs a completed turn, valid events, no provider failure, agent exit code zero, and passed
verification. A repair task also needs a passing recorded check for its final source. Cancellation
takes precedence over a successful process exit. Dropping a worker cancels and joins it. The process
owner stops its child group. The session owner removes private session data after trace capture.

## Comparison and assessment

`TaskId` parses `<group>--<variant>` folder names. Both parts use lowercase letters, numbers, and
single hyphens between words. The full ID is limited to 120 characters. Task discovery and CLI task
arguments reject other names. Run IDs have a separate validator. `tasks --check` validates the task
catalog and input files without opening `Store`. Root verification includes this command.

The shared comparison rule requires two different variants in the exact same group. `Application`
checks it before loading reports. `Comparison` checks the saved report task IDs. Assessment start,
saved assessment access, and assessment recovery enforce it too. Group membership comes from the
saved task ID, so editing task inputs cannot change the group of existing evidence. There is no
separate group configuration, old-name alias, or run migration.

`TaskPair` stores two task IDs for UI selection. `Comparison::Pair` binds two exact task/run IDs.
Measurements use B minus A. Missing values remain missing. Input changes come from saved input
hashes. Setup and final source changes come from saved inventories. The comparison includes both
reports, so provider, model, effort, versions, task settings, and verification remain available.

Cost estimates use API prices in USD. Provider event adapters emit typed usage and cost
observations. The Codex adapter reads saved request traces and checks cumulative totals. Its
fallback uses the requested model and run totals. The Claude adapter uses the final native cost when
present. It also keeps distinct message records, with the last cumulative record for each message
and request. Its CodexBar fallback needs known prices and agreement with final usage when final
usage exists. A partial stream is marked as partial. Missing measurements remain missing.

`Report` stores the amount, basis, models, source, and limits. `Store` only reads saved values. New
reports use schema 2. There is no old-report fallback, migration, or cost cache. TUI rendering does
not read traces. `Comparison` keeps a missing cost difference missing.

The optional assessment takes the execution lock and rechecks both current run IDs before starting.
It uses the selected provider with its assessment permissions. Codex uses its read-only sandbox.
Claude exposes Read, Glob, and Grep only, with access to the two saved run folders. It treats task
prompts, instructions, code, and logs as evidence. It has no browser review step. Its usage never
enters either task report.

There is one assessment folder. Starting an assessment replaces it. Rerunning either member deletes
it. A changed run ID makes it unavailable. Recovery deletes stale assessments and marks abandoned
active assessments as `Interrupted`. Swapped comparisons do not reuse direction-specific text.

## Preview and selection

`Application` stores previews by task ID. Each preview binds the exact current run ID. Starting or
stopping A does not stop B. Polling checks readiness; the interface opens the browser only after
readiness. Quitting stops all owned previews. A preview command in another process holds the same
run lock and blocks replacement until it stops.

The sidebar is a group and task tree. `Tasks` owns group-level and task-level navigation. Group
membership comes from `TaskId`. Enter moves into a group. Esc returns to the group. The tree stays
visible. The saved selection keeps a task ID as the group anchor, the navigation level, the
comparison pair, and whether comparison is open. Older selections start at group level.

Group Overview shows every member, regardless of the sidebar search. Group comparison reuses a valid
saved pair or selects the first two available members. It shows an empty state when there are fewer
than two results. Task comparison uses the selected task as A and opens the group Compare view. A
and B selectors retain their group filter during refresh. Tasks without output remain visible but
cannot form a pair.

`tabs.rs` owns one tab contract for rendering, keyboard input, and mouse targets. Left/Right,
Tab/Shift+Tab, number keys, and mouse clicks all select the same visible tabs. Enter/Esc change the
navigation level. `layout.rs` owns one title, tab row, action row, and content region for all views.

`compare.rs` owns comparison content and actions. It displays configurations, outcomes, and metrics
for both results. Its actions select A, select B, and swap. Task shortcuts and agent actions are not
active in comparison mode. Agent assessment remains a CLI feature. `details.rs` owns task Overview,
Activity, and Setup. Setup separates saved configuration from the current task prompt. Each view
uses shared text formatting from `text.rs`.

On startup, saved selection is checked against the current catalog and available runs. An invalid
pair is cleared and the UI returns to Overview with a message. This includes removed tasks, missing
runs, and pairs from different groups.

## Verification and limits

Automated tests stay in memory. They check replacement state, pair identity, missing evidence,
signed differences, event reduction, lifecycle rules, package settings, and rendering. They do not
create files, start processes, install packages, or open browsers or editors.

Manual checks use an external project and output folder. They check locks, deletion, recovery,
package installation, real provider use, TUI input, browser rendering, and process shutdown.
Compilation and package tooling still write normal build output and caches.

The app runs up to four task runs at once per storage location, one run per task, and holds the
storage lock for its lifetime, so only one app instance uses a storage location at a time. An
assessment does not run while any task run is active or queued. `Application::start_group` validates
all idle members before adding them to `RunQueue`. Existing active or queued members are skipped.
Polling starts pending tasks as slots become free. Dispatch errors remain visible in Group Overview;
worker failures keep their normal reports. Group cancellation removes pending entries and cancels
active members. Quitting drops the queue. It does not resume after restart. There is no event
database, plugin loader, or migration layer. Configuration separation does not provide full host
isolation. Evidence can contain task text and paths. An agent code assessment cannot approve visual
quality.

## Declarative repair setup

`Project` resolves selected profiles before run replacement. `SetupPlan` holds the profile manifest
text and copied file bytes, so execution does not reread profile sources. `PreparedWorkspace`
applies the plan, installs exact dependencies, and saves the resolved setup. The shared quality
module owns root lint and format settings. The profile supplies a small Vite configuration that uses
those settings with standalone source paths and the starter build configuration.

The repair controller starts during setup. It makes a private dependency copy and runs a preflight
check. During the Agent phase, the normal provider output poll also reads repair requests. The
generated `vp run repair` client waits for a response through files in `.agent-ui`. Requests contain
only an ID; they cannot select commands, paths, or environment variables. The controller executes
the configured command vectors against a fresh source snapshot. It records results in memory and in
evidence before returning diagnostics. Responses in the agent workspace are not pass evidence.

Checks use macOS Seatbelt, an empty HOME, explicit tool paths, no network, and write access only to
the check workspace and temporary directory. Installed dependencies stay read-only; Vite caches
point into the private temporary directory. Unsupported platforms reject repair at validation. These
checks are independent of provider-specific hooks and require no Git writes.

Each request has one 300-second limit, capped by the remaining agent time. Cancellation reaches the
check process group. Check failures return to the same agent process. The final check does not start
another agent and cannot count as the required in-pass attempt. The last attempt must pass and match
the complete final source inventory. Setup files outside `src/` and `public/` are fixed. A check
cannot change source or configuration. The final verified build is copied to `app/dist`. The
existing dev preview uses the final source.

The controller excludes only its reserved IPC directory from source evidence, in addition to the
existing dependency, build-output, and Git exclusions. Attempt times cover their command processes;
they are already part of the Agent phase wall time. There is no separate repair token estimate.

Manual acceptance checks must cover fresh setup, a deliberate lint/type/build failure, a corrected
source, skipped repair, a stale pass, altered configuration, command timeout, and cancellation.
Automated parser, policy, and lifecycle tests remain in memory. Provider sandbox integration needs a
separate command-level or live check; fixture provider events are not model execution evidence.
