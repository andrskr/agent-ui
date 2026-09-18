CREATE TABLE batches (
    batch_id TEXT PRIMARY KEY,
    suite TEXT NOT NULL,
    manifest TEXT NOT NULL,
    input_fingerprint TEXT NOT NULL,
    snapshot_json TEXT NOT NULL,
    settings_json TEXT NOT NULL,
    concurrency INTEGER NOT NULL CHECK(concurrency BETWEEN 1 AND 4),
    created_at_ms INTEGER NOT NULL,
    started_at_ms INTEGER,
    finished_at_ms INTEGER,
    recovered_at_ms INTEGER,
    state TEXT NOT NULL CHECK(state IN ('pending','running','completed','incomplete','interrupted'))
);
CREATE TABLE batch_tasks (
    batch_id TEXT NOT NULL REFERENCES batches(batch_id),
    task_id TEXT NOT NULL,
    task_order INTEGER NOT NULL,
    scenario TEXT NOT NULL,
    variant TEXT NOT NULL,
    input_fingerprint TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('pending','active','ready','failed','cancelled','interrupted','failed_to_start')),
    PRIMARY KEY(batch_id, task_id),
    UNIQUE(batch_id, task_order)
);
CREATE TABLE runs (
    run_id TEXT PRIMARY KEY,
    batch_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    attempt_number INTEGER NOT NULL CHECK(attempt_number > 0),
    state TEXT NOT NULL CHECK(state IN ('starting','ready','failed','cancelled','interrupted','failed_to_start')),
    reserved_at_ms INTEGER NOT NULL,
    created_at_ms INTEGER,
    finished_at_ms INTEGER,
    recorded_at_ms INTEGER,
    recovered_at_ms INTEGER,
    provider TEXT NOT NULL,
    model_requested TEXT NOT NULL,
    effort_requested TEXT NOT NULL,
    timeout_seconds INTEGER NOT NULL,
    provider_version TEXT,
    runner_version TEXT,
    runner_build TEXT,
    node_version TEXT,
    vp_version TEXT,
    setup_started_at_ms INTEGER,
    setup_finished_at_ms INTEGER,
    agent_started_at_ms INTEGER,
    verification_started_at_ms INTEGER,
    setup_seconds REAL,
    agent_seconds REAL,
    verification_seconds REAL,
    elapsed_seconds REAL,
    agent_exit_code INTEGER,
    verification_exit_code INTEGER,
    input_tokens INTEGER,
    cached_input_tokens INTEGER,
    cache_write_input_tokens INTEGER,
    output_tokens INTEGER,
    reasoning_output_tokens INTEGER,
    cost_usd REAL,
    cost_basis TEXT,
    cost_source TEXT,
    cost_models_json TEXT,
    changed_file_count INTEGER,
    completed_turns INTEGER,
    invalid_event_lines INTEGER,
    error TEXT,
    report_json TEXT,
    completion_json TEXT,
    FOREIGN KEY(batch_id, task_id) REFERENCES batch_tasks(batch_id, task_id),
    UNIQUE(batch_id, task_id, attempt_number)
);
CREATE INDEX runs_configuration ON runs(provider, model_requested, task_id);
CREATE INDEX runs_recording ON runs(recorded_at_ms);
CREATE TRIGGER immutable_run_update BEFORE UPDATE ON runs
WHEN OLD.recorded_at_ms IS NOT NULL
BEGIN SELECT RAISE(ABORT, 'Completed ledger results are immutable'); END;
CREATE TRIGGER immutable_run_delete BEFORE DELETE ON runs
WHEN OLD.recorded_at_ms IS NOT NULL
BEGIN SELECT RAISE(ABORT, 'Completed ledger results are immutable'); END;
PRAGMA user_version = 3;
PRAGMA application_id = 1096109132;
