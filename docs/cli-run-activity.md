# Permanent run activity

Status: implemented. Recorded CLI batches save metrics, activity, and command output in the same
SQLite database. Inspection and manual review are optional. The app does not generate analytical
reports. Retention and deletion commands are a separate task.

## Commands

```sh
vp run agent-ui batch run --suite ui-evaluation --provider claude --record
vp run agent-ui ledger events <run-id>
vp run agent-ui ledger events <run-id> --json
vp run agent-ui ledger events <run-id> --json --after 200 --limit 100
```

`--record` enables activity capture automatically. A resumed recorded batch also captures new
attempts. Ordinary TUI runs, single-task runs, and batches without `--record` keep their existing
local evidence behavior. They do not add permanent activity.

`ledger events` opens SQLite read-only. It needs no project, provider login, or generated workspace.
It returns up to 200 structured events by default. `--limit` accepts 1 to 1000. `next_after` gives
the cursor for another page when more events exist. During an active run, a later query can find new
events even if the earlier page had no next cursor. Sequence gaps are normal because output chunks
share the sequence with events. Read original output directly from `run_log_chunks`.

## Evidence contract

Each run has an independent increasing sequence. Each record has a UTC observation date and elapsed
milliseconds from a monotonic run clock. These are receipt times, not provider execution times.
Output is polled about every 100 milliseconds; buffering and runner work can increase that interval.
Provider timestamps stay separate. Do not derive exact inference or thinking duration from quiet
intervals or output receipt times.

The recorder saves:

- Run start and capture completion, plus setup, agent, and verification phase boundaries.
- Setup activity notes and each runner-owned command's arguments, working directory, exit status,
  duration, stdout, and stderr. Environment variables and login files are not copied into activity.
- The exact prompt bytes submitted on the provider command's stdin.
- Original provider stdout, including partial deltas, unknown events, and malformed JSON.
- Claude message content, message usage snapshots, tool requests, tool results, stream boundaries,
  provider results, and errors as structured events.

Claude uses `--include-partial-messages`. Its provider adapter owns the event format. Other
providers retain original process output and runner events; they do not yet have Claude's structured
activity mapping. Partial text and thinking deltas remain in original stdout rather than being
duplicated as one SQL event per token. Only content emitted by the provider is available.

`tool.requested` means the runner observed a tool request. It is not an exact tool execution start.
Link a request to its result by `tool_call_id`. A missing result remains missing. Link messages by
`message_id`; stream boundaries inherit the current message ID. The provider events' `command_id`
links them to the provider command and its original stdout. Parent tool IDs stay available when the
provider supplies them. The recorder does not infer hidden retries or work that the provider omits.

Tool work occurs inside the agent phase. Parallel tool intervals can overlap. Cost stays an API
price estimate. There is no exact allocation of run cost to individual tools.

Raw output stays byte-exact, including invalid UTF-8. Command details include readable arguments and
their native representation. Logs can contain prompt text, generated code, and tool output. The
recorder does not scan private authentication files or collect the host environment. It also does
not remove arbitrary sensitive text that a task or tool prints.

This is execution evidence, not a full historical source archive. Generated workspaces remain
latest-per-task. Full code review of an old workspace can require a separate source archive.

## Database schema

Schema 3 includes three activity tables and one view. A run without a `run_capture` row is reported
as `not_recorded`. The runner does not invent historical events.

| Object              | Content                                                                                             |
| ------------------- | --------------------------------------------------------------------------------------------------- |
| `run_capture`       | Capture version, state, last saved sequence, journal offset, raw byte count, and error.             |
| `run_events`        | Ordered event type, phase, command/message/tool IDs, observed and provider times, and JSON details. |
| `run_log_chunks`    | Original stdin/stdout/stderr bytes in chunks of at most 32 KiB, with command ID and byte offset.    |
| `run_request_usage` | Latest Claude usage snapshot per message and request ID.                                            |

Capture state is separate from run state:

| Capture state | Meaning                                                                                |
| ------------- | -------------------------------------------------------------------------------------- |
| `recording`   | Activity is still being collected or awaits recovery.                                  |
| `complete`    | All journal records were committed, with a final capture marker and no capture error.  |
| `partial`     | Execution or capture ended without complete evidence. The saved prefix remains useful. |
| `not_started` | The attempt failed before its activity writer started.                                 |

A failed or cancelled run can have a complete capture. A complete capture means all observed output
was saved; it does not mean every provider-internal action was exposed. A crash can lose bytes not
yet observed by the runner. An incomplete final journal record is reported as a capture error.

`run_request_usage` selects the last cumulative message snapshot. Never sum all `message.usage`
events or add the final provider total to those snapshots. Missing message IDs stay in the event
table but are excluded from this view because they cannot be deduplicated. View input counts use
Claude's categories: `uncached_input_tokens`, `cached_input_tokens`, and `cache_write_input_tokens`.
These differ from `runs.input_tokens`, which includes all three categories. Missing values remain
NULL. The view does not reprice historical requests or assert that partial evidence matches a
complete run total.

## Write and recovery path

```text
Process / Journal / provider adapter
  -> activity-pending/<run-id>.jsonl
  -> one coordinator writes SQLite transactions
  -> final metrics and capture state commit together
  -> remove pending files
```

The coordinator reserves the capture before dispatch. Each worker has one activity writer. It
appends and syncs records outside the replaceable run folder. The coordinator imports small groups
of records during execution. It commits rows and the source offset together. Recovery starts from
that offset, so committed records are not duplicated. Output byte offsets must be contiguous within
each command and stream. Invalid sequences fail recording.

Completion writes the existing pending-result envelope, drains the activity journal, and commits
capture state with the terminal metrics. Completed activity is immutable. A write failure stops new
dispatch and cancels active workers. Pending evidence remains until recording succeeds. Startup
holds the storage lock before replay and before ordinary run-folder cleanup. Interrupted runs seal
their saved activity as partial. Resume creates new attempts and new activity records.

There is no automatic expiry, silent output truncation, or size-based deletion. `log_bytes` records
raw output size, not total SQLite file size. Retention can later remove logs independently of
metrics through an explicit supported operation. Current SQL triggers prevent ordinary deletion.

## Example queries

Read the ordered activity for one attempt:

```sql
SELECT sequence, elapsed_ms, phase, kind, message_id, tool_call_id, details_json
FROM run_events
WHERE run_id = :run_id
ORDER BY sequence;
```

Find requests with large input usage. NULL cache values remain visible:

```sql
SELECT message_id, model, uncached_input_tokens, cached_input_tokens,
       cache_write_input_tokens, output_tokens
FROM run_request_usage
WHERE run_id = :run_id
ORDER BY uncached_input_tokens DESC;
```

Find tool results that report errors:

```sql
SELECT tool_call_id, elapsed_ms, details_json
FROM run_events
WHERE run_id = :run_id AND kind = 'tool.result'
  AND json_extract(details_json, '$.is_error') = 1
ORDER BY sequence;
```

Read one command's output as bytes. Concatenate the returned BLOBs in this order:

```sql
SELECT byte_offset, content
FROM run_log_chunks
WHERE run_id = :run_id AND command_id = :command_id AND stream = 'stderr'
ORDER BY byte_offset;
```

## Validation

In-memory tests cover sequence and byte-offset rules, split journal records, transaction rollback,
replay, immutable completion, partial captures, usage deduplication, and schema upgrades. Provider
fixtures cover tool/result identity, unknown events, invalid events, and stream message identity.
Separate process checks cover byte-exact output, large stderr, task replacement, failed attempts,
retries, cancellation, abrupt shutdown, pending write recovery, and the read-only CLI. Process
checks use synthetic provider output; they do not establish real model latency or real billing
costs.
