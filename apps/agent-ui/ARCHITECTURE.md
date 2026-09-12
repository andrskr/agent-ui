# Architecture

The app runs task experiments and lets a person compare their code and evidence. CLI and TUI use the
same application service. Each task has at most one current run. Comparison joins two different
tasks. There is no run history, approval state, or promotion step.

## Owners

| Owner               | Responsibility                                                       |
| ------------------- | -------------------------------------------------------------------- |
| `Project`           | Find tasks and validate source inputs before replacement.            |
| `TaskConfig`        | Parse TOML and transform package settings in memory.                 |
| `Application`       | Coordinate runs, assessments, and previews for both interfaces.      |
| `Store`             | Own the task index, run files, locks, replacement, and recovery.     |
| `Index`             | Enforce current and removing slots in memory.                        |
| `Runner`            | Coordinate setup, agent execution, and verification.                 |
| `PreparedWorkspace` | Copy inputs and starter, install packages, and record source hashes. |
| `Journal`           | Measure phases and save progress and command output.                 |
| `Report`            | Apply task lifecycle and completion rules to typed observations.     |
| `Comparison`        | Compute signed differences from two saved results.                   |
| `Assessment`        | Run an explicit Codex comparison and record its separate evidence.   |
| `Worker<T>`         | Own cancellation and join a background thread.                       |
| `Preview`           | Own one server, port, lock, and readiness state.                     |
| `Process`           | Own child groups and capture output.                                 |
| `Codex`             | Own private session setup, command arguments, decoding, and traces.  |
| TUI                 | Own selection, forms, search, scroll, and rendering.                 |

The TUI does not read evidence files or own child processes. Pure comparison and rendering do not
start Codex. `toolchain.rs` resolves executables; it does not select tasks or start previews.

## Replacement

1. Validate settings, task source, package settings, starter manifest, and executables.
2. Take the storage execution lock. No other run or assessment can start.
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
verification. Cancellation takes precedence over a successful process exit. Dropping a worker
cancels and joins it. The process owner stops its child group. The session owner removes private
session data after trace capture.

## Comparison and assessment

`TaskPair` stores two task IDs for UI selection. `Comparison::Pair` binds two exact task/run IDs.
Measurements use B minus A. Missing values remain missing. Input changes come from saved input
hashes. Setup and final source changes come from saved inventories. The comparison includes both
reports, so model, effort, versions, task settings, and verification remain available.

The optional assessment takes the execution lock and rechecks both current run IDs before starting.
It uses a fresh Codex session with a read-only sandbox. It treats task prompts, instructions, code,
and logs as evidence. It has no browser review step. Its usage never enters either task report.

There is one assessment folder. Starting an assessment replaces it. Rerunning either member deletes
it. A changed run ID makes it unavailable. Recovery deletes stale assessments and marks abandoned
active assessments as `Interrupted`. Swapped comparisons do not reuse direction-specific text.

## Preview and selection

`Application` stores previews by task ID. Each preview binds the exact current run ID. Starting or
stopping A does not stop B. Polling checks readiness; the interface opens the browser only after
readiness. Quitting stops all owned previews. A preview command in another process holds the same
run lock and blocks replacement until it stops.

The sidebar selection is a task ID. The comparison pair and whether comparison is open are saved.
The selected side chooses Activity, Evidence, code, and preview actions. Overview displays both
results. The picker includes tasks without output, but they cannot form a comparison yet.

## Verification and limits

Automated tests stay in memory. They check replacement state, pair identity, missing evidence,
signed differences, event reduction, lifecycle rules, package settings, and rendering. They do not
create files, start processes, install packages, or open browsers or editors.

Manual checks use an external project and output folder. They check locks, deletion, recovery,
package installation, real Codex use, TUI input, browser rendering, and process shutdown.
Compilation and package tooling still write normal build output and caches.

The app has one active operation per storage location. It has no job queue, event database, provider
framework, or migration layer. Configuration separation does not provide full host isolation.
Evidence can contain task text and paths. A Codex code assessment cannot approve visual quality.
