# CLI batch execution and results ledger

Status: implemented contract. See [the usage and database guide](cli-ledger.md).

## Purpose and scope

Run a named suite with one provider and model. Save each result to a permanent local SQLite
database. Later, a person or agent can query that database to prepare a report.

A suite with 12 scenarios and two variants selects 24 tasks. Each task runs once in a new batch. A
second batch adds new records. It does not replace earlier records.

The feature includes suite selection, batch execution, automatic recording, and restart support.
Comparison commands, report generation, exports, manual review states, and database screens in the
TUI are outside this feature. Existing comparison behaviour is unchanged.

Manual inspection is optional. It does not affect execution or recording. The existing TUI can open
the latest generated code and preview after the CLI releases the storage lock.

## Terms and retained data

| Term     | Definition                                                         |
| -------- | ------------------------------------------------------------------ |
| Scenario | The existing task group, such as `recent-transactions`.            |
| Variant  | The existing variant, such as `baseline` or `context`.             |
| Suite    | A named, explicit selection of scenarios and variants.             |
| Batch    | One execution of the resolved suite with one configuration.        |
| Run      | One attempt to execute one task. Each attempt has a unique run ID. |
| Ledger   | The database of batch plans, attempts, and completed measurements. |

Each task still has one current set of generated artifacts. Normal replacement removes its old code,
logs, and preview files. Ledger records survive that removal. The ledger cannot reopen an
application whose files have been removed.

Keep completed result records immutable. A retry creates a new attempt. Batch progress and pending
work can change. Do not add approval or promotion states.

## Suite file

Store suites at `experiments/suites/<name>.toml`. Use the existing lowercase identifier rules. The
following example selects two tasks from the Recent transactions scenario:

```toml
schema_version = 1
scenarios = ["recent-transactions"]
variants = ["baseline", "context"]
```

Resolve the cross product in file order: scenario first, then variant. Require every selected task
to exist. Reject empty lists, duplicate entries, unknown fields, and invalid identifiers. Do not
silently omit a missing task. A task added outside the explicit selection does not enter the suite.

Before execution, resolve and validate every task, its settings, starter, and setup profiles. Save
the exact selection and a content fingerprint. Include task inputs, starter source, lockfiles,
resolved profile files, and verification definitions in the fingerprint. Exclude build output,
installed dependencies, and Git metadata. A Git commit alone is not a content fingerprint.

For recorded batches, save an input snapshot in SQLite, outside replaceable run folders. Resume from
this snapshot. Do not reread changed source files as if they were the original batch inputs.
Installed packages and tool binaries are not archived. Record their resolved versions with each
attempt.

## CLI contract

Run these commands from the repository root.

```sh
# Run every task in the suite and save every result.
vp run agent-ui batch run --suite recent-transactions --provider claude --model claude-sonnet-5 --record

# Check the resolved selection without running agents or replacing output.
vp run agent-ui batch run --suite recent-transactions --provider claude --model claude-sonnet-5 --dry-run

# Inspect execution and recording status.
vp run agent-ui batch show <batch-id>

# Recover records, then execute tasks that have never started.
vp run agent-ui batch resume <batch-id>

# Also retry failed, cancelled, interrupted, or failed-to-start tasks.
vp run agent-ui batch resume <batch-id> --retry-incomplete

# Print the database path and schema version for later SQL access.
vp run agent-ui ledger info
```

The included `recent-transactions` suite selects the two Recent transactions tasks. Extend its
explicit scenario list as new tasks are added. There are not yet 12 scenarios in this suite.

| Argument                              | Behaviour                                                                                                                     |
| ------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `--suite <name>`                      | Required for a new batch. Select one suite file.                                                                              |
| `--provider <id>`                     | Use the existing provider selector. `claude` selects Anthropic.                                                               |
| `--model <id>`                        | Use the existing model selector and validation. Save the resolved request.                                                    |
| `--effort <value>`                    | Optional. Omission uses the local model catalog default.                                                                      |
| `--timeout <seconds>`                 | Existing per-task agent timeout. Default remains 900 seconds. It is not a batch deadline.                                     |
| `--record`                            | Save the batch plan and every result to SQLite.                                                                               |
| `--dry-run`                           | Validate and print task IDs, count, configuration, recording mode, and replacement scope. No mutations or provider execution. |
| `--retry-incomplete`                  | On resume, add one new attempt for each task whose latest attempt did not reach Ready.                                        |
| `--project`, `--data-dir`, `--binary` | Preserve the existing global argument behaviour.                                                                              |

`--effort default` delegates effort selection to the provider CLI. Save that request literally. Do
not invent an effective effort if the provider does not report it. An omitted model or provider
continues to use existing defaults; the printed plan must make those choices clear.

A batch without `--record` uses a temporary queue and current run reports. It has no permanent batch
record or resume support. Recording does not depend on run success. There is no save dialog.

Resume keeps the saved task inputs, provider, model, effort, and timeout. Reject overrides that
would change this configuration. To change configuration, start a new batch. Successful tasks are
never rerun by resume. Retrying does not change any previous attempt or its measurements.

Print the batch ID and database path at the start of a recorded batch. Send progress to stderr. On
exit, print a JSON execution summary to stdout with the batch ID, task counts, attempt counts,
states, and recording errors. `batch show` also returns JSON. These are execution status outputs,
not analytical reports.

Use exit code 0 only when every selected task has reached Ready and every required record is
durable. Use 1 for failed or incomplete execution and storage errors, 2 for invalid arguments or
plans, and 130 for Ctrl+C. Failure states in the database remain more specific than the exit code.

## Execution and recovery

Reuse `Application`, `Runner`, and the existing queue. Keep at most four active tasks per storage
location and one active attempt per task. Save the concurrency limit in the batch configuration.
Keep the existing single-process storage lock. The first version does not add a background daemon.

1. Validate all inputs, executable paths, provider version, and login before replacing output.
2. For recording, verify that SQLite is writable and its schema is supported.
3. Save the input snapshot, batch configuration, and complete pending task list before dispatch.
4. Reserve a run ID and attempt number before starting each task. Record start failures explicitly.
5. Use the normal current-artifact replacement and run lifecycle.
6. Save the terminal run report, then commit the result and task progress in one database
   transaction.
7. Continue other tasks after an individual task failure. Do not retry automatically.

Ctrl+C stops new dispatch, cancels active process groups, and records their partial reports. Tasks
that never started remain pending. Do not label pending work as failed model execution.

If a ledger write fails, stop new dispatch. Let active workers stop safely and preserve their
reports. Exit unsuccessfully. Keep durable recording intents outside replaceable run folders,
including the run ID, batch membership, and report location or terminal snapshot.

On startup, reconcile these intents before normal cleanup can remove their evidence. Replaying the
same run ID with the same terminal snapshot is a no-op. A conflicting snapshot is an error. Never
overwrite a completed ledger entry. Prevent all replacement paths, including the TUI, from deleting
a result that is still waiting to be recorded.

Recover an abandoned active attempt as interrupted if no terminal report exists. Leave unknown
measurements NULL. A recovery timestamp is not the actual crash time; store it separately and leave
the true finish time unknown. `batch resume` first completes recording recovery, then starts pending
work. It retries unsuccessful attempts only with `--retry-incomplete`.

## Database contract

Store `ledger.sqlite3` directly under the selected data directory, outside `runs/`. The default
location is `~/Library/Application Support/Agent UI/ledger.sqlite3`. Normal artifact cleanup must
never delete the database or recorded batch input snapshots.

Use SQLite foreign keys, explicit transactions, unique constraints, and a versioned schema.
Serialize writes through the application owner. Query tools can open the database read-only. Fail
safely on a newer unsupported schema. Preserve recorded data during future schema upgrades. There is
no import or migration of old run reports in the first version.

Use these logical tables. Final DDL must retain these relationships and field meanings.

| Table         | Key and contents                                                                                                                      |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `batches`     | `batch_id`; suite name and manifest; input fingerprint and snapshot JSON; configuration; created/start/finish times; execution state. |
| `batch_tasks` | `(batch_id, task_id)`; explicit task order; scenario; variant; input fingerprint; current scheduling state.                           |
| `runs`        | `run_id`; batch/task foreign key; attempt number; lifecycle state; dates; metrics; configuration and versions; terminal report JSON.  |

Use a unique constraint on `(batch_id, task_id, attempt_number)`. Allocate attempt numbers in a
transaction. A retry gets a new `run_id`. `runs` initially holds an attempt identity and execution
state. Once a terminal report is committed, its evidence becomes immutable. Failed-to-start attempts
can have no runner report; keep their error and missing measurements explicit.

Give frequently queried values typed columns. Preserve a versioned terminal report JSON snapshot for
supporting evidence. Keep cost model IDs, cost provenance, warnings, input hashes, source
inventories, changed-file paths, and resolved setup in that snapshot. SQL queries must not require
the generated code or original report file. Snapshot file paths are historical references, not a
promise that artifacts still exist.

Use UTC Unix milliseconds for dates, REAL seconds for durations, INTEGER for token counts, and
nullable fields for unavailable evidence. Store the provider's reported cost value without display
rounding. Label it USD API estimate. Store its basis and source too. Never substitute current prices
when reading an old record.

## Measurements

These values come from the current run report unless marked as new or derived.

| Field or group                                        | Meaning and rules                                                                                                         |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `run_id`, `task_id`, scenario, variant                | Exact run and task identity. Scenario and variant come from the task ID.                                                  |
| `batch_id`, suite, fingerprint                        | New batch identity and saved input version.                                                                               |
| Provider, requested model, requested effort           | Exact configuration submitted to the runner. Preserve reported cost model IDs separately.                                 |
| Timeout and tool versions                             | Timeout in seconds; provider CLI, runner, Node, and Vite+ versions. Add a runner build/source identity where available.   |
| Created, started, finished, recorded, recovered dates | Distinguish lifecycle dates, ledger commit time, and recovery time. New fields must not be inferred from unrelated dates. |
| Run state and error                                   | Ready, failed, cancelled, interrupted, or failed to start. Ready means current automated completion rules passed.         |
| `setup_seconds`                                       | Measured setup duration. Distinguish a completed phase from partial or unstarted setup.                                   |
| `agent_seconds`                                       | Measured agent duration. It includes tool work.                                                                           |
| `verification_seconds`, exit code                     | Measured final verification and its result. NULL if not available.                                                        |
| `elapsed_seconds`                                     | Derived from real run creation and finish timestamps. Includes runner overhead. Unknown for an unobserved crash.          |
| `input_tokens`                                        | Normalized provider input usage. For current Claude records, includes cache reads and writes.                             |
| `cached_input_tokens`                                 | Cache-read usage. A subset of normalized input tokens, not an additional total.                                           |
| `cache_write_input_tokens`                            | Cache-write usage when reported. Do not add it again to normalized input tokens.                                          |
| `output_tokens`                                       | Provider output usage.                                                                                                    |
| `reasoning_output_tokens`                             | Separate reasoning usage when reported. Do not invent it or add overlapping categories.                                   |
| `cost_usd`, basis, source, model IDs                  | Saved API estimate and evidence. Not a subscription charge. Missing cost is NULL, not zero.                               |
| Changed files                                         | Saved paths and derived count. This is not a quality score. Missing source evidence must not imply zero changes.          |
| Completed turns, invalid event lines, event counts    | Existing diagnostic evidence. Provider event counts are not a portable tool-call metric.                                  |
| Warnings, agent exit code, final verification         | Preserve failure and partial-result evidence.                                                                             |

Retain evidence completeness and phase state. Do not convert an unstarted or unknown phase into a
zero duration just because an old report field starts at zero. No visual quality score, browser pass
result or manual review status is collected by this feature.

## Implementation boundaries

Add a suite resolver next to task discovery, a persistent batch plan owned through `Application`,
and a ledger owner for SQLite transactions. Keep provider event parsing in provider adapters. Keep
measurement rules in typed report code. The CLI must not calculate provider usage or cost.

Use one recording path for normal completion, cancellation, start failure, and recovery. Preserve
the existing unrecorded single-task and TUI workflows. Add only the recovery protection needed to
prevent deletion of unsaved recorded evidence. No new TUI screens are required.

When implementing, update the current architecture and package instructions that say no history or
persistent queue exists. Limit that change to recorded batch metadata and measurements. Keep one
current artifact set per task and the existing comparison group rule.

## Acceptance checks

1. A valid 12-by-3 suite resolves to exactly 36 unique tasks. Missing variants fail before mutation.
2. A dry run starts no provider and creates no database, snapshot, or run output.
3. A recorded batch saves successes and failures as each attempt finishes.
4. A second model batch preserves all first-batch records while replacing current artifacts.
5. Duplicate delivery of a completed result creates no duplicate record. Conflicting evidence fails.
6. Ctrl+C preserves completed and partial evidence. Pending tasks remain distinguishable.
7. Recovery works across the report-save/database-commit boundary and protects unsaved evidence.
8. Resume uses saved inputs. It starts pending tasks only unless retries are explicitly requested.
9. Retries get new attempt IDs and preserve all earlier results. Successful tasks are not repeated.
10. A database write failure prevents new dispatch and reports an unsuccessful command result.
11. Missing metrics remain NULL. Cache categories are not counted twice.
12. Read-only SQL can retrieve all saved metrics after generated artifacts have been replaced.
13. Optional TUI/browser inspection has no effect on recording or completion.

Test parsing, scheduling, lifecycle transitions, constraints, and SQL queries with in-memory data
and an in-memory SQLite database. Use independent expected values. Do not create files or child
processes in automated tests. Check real interruption, locking, replacement, and crash recovery
manually in a separate data directory. Run the package and root verification commands before
implementation handoff.
