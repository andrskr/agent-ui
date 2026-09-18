# CLI results ledger

The CLI records measurements and run activity in a local SQLite database. It does not generate
analytical reports. Later, open the database read-only to answer questions or prepare a report.

## Run a suite

```sh
vp run agent-ui batch run --suite ui-evaluation --provider claude --model claude-sonnet-5 --dry-run
vp run agent-ui batch run --suite ui-evaluation --provider claude --model claude-sonnet-5 --record
```

The `ui-evaluation` suite selects Invite member and its three variants. The `project-list` suite
selects Project list and its three variants. Use `--suite project-list --timeout 1800` for a
30-minute agent timeout per task. Suite files live in `experiments/suites/`:

```toml
schema_version = 1
scenarios = ["invite-member"]
variants = ["baseline", "context", "repair"]
```

The `notification-preferences` suite selects Notification preferences and the same three variants.
Use `--suite notification-preferences --timeout 1800` to run that scenario with a 30-minute agent
timeout per task.

Each scenario/variant pair must exist. No wildcard expands the selection. Names, duplicates, empty
lists, and unknown fields are checked before execution. Without `--record`, a batch has no permanent
ledger or resume support. `--dry-run` creates no storage and starts no provider process.

Effort uses the current local model default when omitted. `--effort default` lets the provider CLI
choose instead. The requested value is saved; an unreported effective effort remains unknown. The
default agent timeout is 900 seconds per task. Up to four tasks execute at once.

## Inspect and resume execution

```sh
vp run agent-ui batch show <batch-id>
vp run agent-ui batch resume <batch-id>
vp run agent-ui batch resume <batch-id> --retry-incomplete
vp run agent-ui ledger info
```

`batch show` prints counts and the latest scheduling state for each task. `batch resume` uses the
original input snapshot and configuration. It starts pending work. Add `--retry-incomplete` to also
retry unsuccessful tasks. Each retry gets a new attempt number and run ID. Successful tasks are
never repeated by resume. Configuration overrides are rejected.

A changed original prompt does not affect a resumed batch. Start a new batch to use new inputs or
another model. Tool binaries and installed dependencies are not archived. Each run saves the tool
versions it observes. A runner execution-contract change can require a new batch.

The CLI prints progress to stderr and a final JSON execution summary to stdout. A recording error
stops new dispatch and cancels active workers. Their saved evidence remains available for recovery.
Exit codes: 0 means all tasks reached Ready and recording succeeded; 1 means incomplete execution or
a storage error; 2 means an invalid command or plan; 130 means Ctrl+C.

The CLI holds the same storage lock as the TUI. Open the TUI after the batch ends to inspect the
latest generated code. Inspection is optional. It does not change the ledger.

## Location and schema

By default, the database is:

```text
~/Library/Application Support/Agent UI/ledger.sqlite3
```

`--data-dir` selects another directory outside the project. `ledger info` reports its path and
schema version without creating a database. Schema version 2 keeps the four result tables below and
adds [permanent activity tables](cli-run-activity.md):

| Table             | One row represents                              | Key                       |
| ----------------- | ----------------------------------------------- | ------------------------- |
| `batches`         | A saved suite selection and configuration       | `batch_id`                |
| `batch_tasks`     | A selected task and its latest scheduling state | `batch_id`, `task_id`     |
| `runs`            | An execution attempt and its recorded result    | `run_id`                  |
| `repair_attempts` | A recorded in-pass repair check                 | `run_id`, `attempt_index` |

The exact definitions are in [ledger.sql](../apps/agent-ui/src/ledger.sql). Foreign keys link the
tables. `(batch_id, task_id, attempt_number)` is unique. Completed result rows and repair evidence
cannot be updated or deleted through normal SQL mutations. Pending scheduling state can change.
There is no import of earlier report files.

`batches.snapshot_json` contains the suite manifest, task input bytes, starter source, and resolved
setup plans. Its fingerprint is in `input_fingerprint`. `settings_json` contains the provider,
requested model and effort, timeout, and optional binary override. Input snapshots are part of the
database so it remains sufficient for later queries. They can make the database larger than a
metrics-only file.

`runs.report_json` preserves the complete terminal report. It includes warnings, event counts, input
hashes, before/after source inventories, changed paths, task settings, and cost evidence.
`completion_json` is the recording envelope used to detect duplicate or conflicting delivery.
Historical artifact paths in these documents can refer to files that no longer exist. Schema 2
records activity and raw command output separately. See [run activity](cli-run-activity.md) for
capture states, recovery, byte output, and request-level usage queries.

## Field meanings

| Fields                                                      | Units and rules                                                                                                                    |
| ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `reserved_at_ms`                                            | UTC Unix milliseconds when the attempt was reserved. No agent need have started.                                                   |
| `created_at_ms`, `finished_at_ms`                           | Real run creation and observed terminal times, in UTC Unix milliseconds.                                                           |
| `recorded_at_ms`                                            | UTC Unix milliseconds when the terminal result was committed. NULL means recording is incomplete.                                  |
| `recovered_at_ms`                                           | Time recovery detected an abandoned run. It is not its actual finish time.                                                         |
| `setup_started_at_ms`, `setup_finished_at_ms`               | Setup phase start and successful completion. A missing completion can indicate partial setup.                                      |
| `agent_started_at_ms`, `verification_started_at_ms`         | Observed phase starts.                                                                                                             |
| `state`                                                     | `starting`, `ready`, `failed`, `cancelled`, `interrupted`, or `failed_to_start`.                                                   |
| `setup_seconds`, `agent_seconds`, `verification_seconds`    | Measured durations. Unobserved phases are NULL. Interrupted measurements can be partial.                                           |
| `elapsed_seconds`                                           | Actual finish minus run creation. Includes overhead outside measured phases. NULL after an unobserved crash.                       |
| `input_tokens`                                              | Normalized input usage. Current Claude input totals include cache reads and writes.                                                |
| `cached_input_tokens`, `cache_write_input_tokens`           | Cache usage already included in normalized input totals. Do not add it again.                                                      |
| `output_tokens`, `reasoning_output_tokens`                  | Output usage and reported reasoning subset. Missing reasoning usage is NULL.                                                       |
| `cost_usd`, `cost_basis`, `cost_source`, `cost_models_json` | Saved API price estimate and provenance. It is not a subscription charge.                                                          |
| `agent_exit_code`, `verification_exit_code`                 | Observed command exit codes. NULL if unavailable.                                                                                  |
| `repair_check_count`                                        | Number of recorded checks, not number of fixes.                                                                                    |
| `changed_file_count`                                        | Count from final source evidence. NULL if that evidence is missing.                                                                |
| `completed_turns`, `invalid_event_lines`                    | Provider event diagnostics. They do not measure visual quality.                                                                    |
| Provider, model, effort, timeout and version columns        | Exact requested settings and observed tool versions. `runner_build` is optional and comes from the build-time `AGENT_UI_BUILD_ID`. |

Never replace missing measurements with zero. An interrupted run can retain usage and cost measured
before interruption. Read its state and cost note before treating those values as complete. Repair
check durations are already inside agent duration. Do not add them twice.

`ready` means the existing automated run checks passed. It does not certify visual quality or imply
a manual review. No manual review fields are recorded.

## Read-only SQL examples

Open the path printed by `ledger info` with a SQLite client in read-only mode. For example:

```sh
sqlite3 -readonly "$HOME/Library/Application Support/Agent UI/ledger.sqlite3"
```

List individual recorded results, including unsuccessful ones:

```sql
SELECT r.run_id, r.batch_id, t.scenario, t.variant,
       r.attempt_number, r.model_requested, r.state,
       datetime(r.created_at_ms / 1000.0, 'unixepoch') AS created_utc,
       r.agent_seconds, r.input_tokens, r.output_tokens, r.cost_usd
FROM runs AS r
JOIN batch_tasks AS t USING (batch_id, task_id)
WHERE r.recorded_at_ms IS NOT NULL
ORDER BY r.reserved_at_ms, r.run_id;
```

Read the raw evidence for one result:

```sql
SELECT report_json FROM runs WHERE run_id = 'replace-with-run-id';
```

Select each task's latest attempt in one batch while retaining older attempts in the database:

```sql
SELECT r.* FROM runs AS r
WHERE r.batch_id = 'replace-with-batch-id'
  AND r.attempt_number = (
    SELECT max(a.attempt_number) FROM runs AS a
    WHERE a.batch_id = r.batch_id AND a.task_id = r.task_id
  );
```

Use explicit attempt selection when preparing a later report. A retry does not remove an earlier
failure. These examples read stored data; the application has no report or comparison command for
the ledger.

## Recovery and artifact replacement

Run IDs are reserved in SQLite before workers start. The terminal report is saved before its
measurements are committed. A durable `ledger-pending/` outbox covers interrupted delivery. Startup
recovers the outbox and unfinished reservations before ordinary artifact cleanup. Identical result
delivery is safe to repeat. Conflicting evidence is an error.

All artifact deletion paths block results whose recording is incomplete. If SQLite is unavailable,
fix that condition and restart. Recovery must complete before protected artifacts can be replaced.
Batch input materializations under `batch-work/` are temporary; the database contains the original
snapshots. Ordinary task replacement never deletes the ledger.

## Validation

Automated tests use in-memory data and SQLite only. They cover suite parsing, exact selection,
snapshot identity, retries, immutable results, transaction rollback, missing measurements, and
recorded numeric values. Filesystem, interruption, lock, and process checks use a separate manual
scratch project and data directory. Fixture provider output is not evidence of a real model run.
