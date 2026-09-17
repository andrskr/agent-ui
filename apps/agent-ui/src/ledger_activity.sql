CREATE TABLE run_capture (
    run_id TEXT PRIMARY KEY REFERENCES runs(run_id),
    capture_version INTEGER NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('recording','complete','partial','not_started')),
    last_sequence INTEGER NOT NULL DEFAULT 0,
    journal_offset INTEGER NOT NULL DEFAULT 0,
    log_bytes INTEGER NOT NULL DEFAULT 0,
    error TEXT
);
CREATE TABLE run_events (
    run_id TEXT NOT NULL REFERENCES run_capture(run_id),
    sequence INTEGER NOT NULL,
    observed_at_ms INTEGER NOT NULL,
    elapsed_ms INTEGER NOT NULL,
    kind TEXT NOT NULL,
    phase TEXT NOT NULL,
    command_id TEXT,
    message_id TEXT,
    tool_call_id TEXT,
    parent_tool_call_id TEXT,
    provider_at_ms INTEGER,
    details_json TEXT NOT NULL,
    PRIMARY KEY(run_id, sequence)
);
CREATE TABLE run_log_chunks (
    run_id TEXT NOT NULL REFERENCES run_capture(run_id),
    sequence INTEGER NOT NULL,
    observed_at_ms INTEGER NOT NULL,
    elapsed_ms INTEGER NOT NULL,
    command_id TEXT NOT NULL,
    stream TEXT NOT NULL,
    byte_offset INTEGER NOT NULL,
    content BLOB NOT NULL,
    PRIMARY KEY(run_id, sequence),
    UNIQUE(run_id, command_id, stream, byte_offset)
);
CREATE INDEX run_events_kind ON run_events(run_id, kind, sequence);
CREATE INDEX run_events_tool ON run_events(run_id, tool_call_id, sequence);
CREATE VIEW run_request_usage AS
SELECT run_id, message_id, request_id, model, sequence,
    json_extract(details_json,'$.usage.input_tokens') AS uncached_input_tokens,
    json_extract(details_json,'$.usage.cache_read_input_tokens') AS cached_input_tokens,
    json_extract(details_json,'$.usage.cache_creation_input_tokens') AS cache_write_input_tokens,
    json_extract(details_json,'$.usage.output_tokens') AS output_tokens
FROM (
    SELECT *, json_extract(details_json,'$.request_id') AS request_id,
        json_extract(details_json,'$.model') AS model,
        row_number() OVER (PARTITION BY run_id,message_id,coalesce(json_extract(details_json,'$.request_id'),'') ORDER BY sequence DESC) AS latest
    FROM run_events WHERE kind='message.usage' AND message_id IS NOT NULL
) WHERE latest=1;
CREATE TRIGGER sealed_capture_update BEFORE UPDATE ON run_capture
WHEN OLD.state<>'recording'
BEGIN SELECT RAISE(ABORT,'Recorded activity is immutable'); END;
CREATE TRIGGER sealed_capture_delete BEFORE DELETE ON run_capture
BEGIN SELECT RAISE(ABORT,'Recorded activity is immutable'); END;
CREATE TRIGGER sealed_event_insert BEFORE INSERT ON run_events
WHEN (SELECT state FROM run_capture WHERE run_id=NEW.run_id)<>'recording'
BEGIN SELECT RAISE(ABORT,'Recorded activity is immutable'); END;
CREATE TRIGGER sealed_event_update BEFORE UPDATE ON run_events
BEGIN SELECT RAISE(ABORT,'Recorded activity is immutable'); END;
CREATE TRIGGER sealed_event_delete BEFORE DELETE ON run_events
BEGIN SELECT RAISE(ABORT,'Recorded activity is immutable'); END;
CREATE TRIGGER sealed_log_insert BEFORE INSERT ON run_log_chunks
WHEN (SELECT state FROM run_capture WHERE run_id=NEW.run_id)<>'recording'
BEGIN SELECT RAISE(ABORT,'Recorded activity is immutable'); END;
CREATE TRIGGER sealed_log_update BEFORE UPDATE ON run_log_chunks
BEGIN SELECT RAISE(ABORT,'Recorded activity is immutable'); END;
CREATE TRIGGER sealed_log_delete BEFORE DELETE ON run_log_chunks
BEGIN SELECT RAISE(ABORT,'Recorded activity is immutable'); END;
PRAGMA user_version=2;
