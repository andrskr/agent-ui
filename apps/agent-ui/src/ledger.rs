use crate::{
    report::{Report, now},
    settings::Settings,
    storage::{Store, private_dir, valid_run_id, write_json},
    suite::{Snapshot, fingerprint},
    task::TaskId,
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OpenFlags, OptionalExtension, named_params, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

const SCHEMA: i64 = 2;
const APPLICATION_ID: i64 = 1096109132;

pub(crate) struct Ledger {
    pub(crate) db: Connection,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Completion {
    pub run_id: String,
    pub task: String,
    pub state: String,
    pub error: Option<String>,
    pub recovered_at_ms: Option<u64>,
    pub report: Option<Report>,
}
impl Completion {
    pub fn report(report: Report) -> Result<Self> {
        ensure!(!report.state.active(), "Cannot record an active run");
        Ok(Self {
            run_id: report.id.clone(),
            task: report.task.clone(),
            state: serde_json::to_value(report.state)?
                .as_str()
                .context("Invalid run state")?
                .into(),
            error: report.error.clone(),
            recovered_at_ms: report.recovered_at_ms,
            report: Some(report),
        })
    }
    pub fn no_report(run_id: String, task: String, error: String, recovered: bool) -> Self {
        Self {
            run_id,
            task,
            state: if recovered {
                "interrupted"
            } else {
                "failed_to_start"
            }
            .into(),
            error: Some(error),
            recovered_at_ms: recovered.then(now),
            report: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BatchSummary {
    pub batch_id: String,
    pub suite: String,
    pub state: String,
    pub planned: usize,
    pub task_states: BTreeMap<String, usize>,
    pub attempts: usize,
    pub recorded: usize,
    pub tasks: Vec<TaskStatus>,
}
#[derive(Debug, Serialize)]
pub struct TaskStatus {
    pub task: String,
    pub state: String,
}
impl BatchSummary {
    pub fn successful(&self) -> bool {
        self.task_states.get("ready").copied().unwrap_or(0) == self.planned
            && self.attempts == self.recorded
    }
}

pub(crate) fn path(root: &Path) -> PathBuf {
    root.join("ledger.sqlite3")
}

impl Ledger {
    pub fn temporary() -> Result<Self> {
        Self::initialize(Connection::open_in_memory()?, true)
    }
    pub fn open(root: &Path, create: bool) -> Result<Self> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | if create {
                OpenFlags::SQLITE_OPEN_CREATE
            } else {
                OpenFlags::empty()
            };
        let db =
            Connection::open_with_flags(path(root), flags).context("Cannot open results ledger")?;
        Self::initialize(db, create)
    }
    fn initialize(mut db: Connection, create: bool) -> Result<Self> {
        db.busy_timeout(Duration::from_secs(2))?;
        db.execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL;")?;
        let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version == 0 && create {
            let count: i64 = db.query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get(0),
            )?;
            ensure!(count == 0, "Refusing to initialize an unrelated database");
            let tx = db.transaction()?;
            tx.execute_batch(include_str!("ledger.sql"))?;
            tx.commit()?;
        }
        Self::check_schema(&db)?;
        let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version == 1 {
            let tx = db.transaction()?;
            tx.execute_batch(include_str!("ledger_activity.sql"))?;
            tx.commit()?;
        }
        Ok(Self { db })
    }
    pub(crate) fn check_schema(db: &Connection) -> Result<()> {
        let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        let application: i64 = db.query_row("PRAGMA application_id", [], |r| r.get(0))?;
        ensure!(
            (1..=SCHEMA).contains(&version) && application == APPLICATION_ID,
            "Unsupported ledger schema or database identity (version {version})"
        );
        Ok(())
    }
    pub fn info(root: &Path) -> Result<serde_json::Value> {
        let file = path(root);
        if !file.exists() {
            return Ok(serde_json::json!({"path":file,"exists":false,"schema_version":null}));
        }
        let db = Connection::open_with_flags(&file, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Self::check_schema(&db)?;
        let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        Ok(serde_json::json!({"path":file,"exists":true,"schema_version":version}))
    }
    pub fn read_summary(root: &Path, id: &str) -> Result<BatchSummary> {
        valid_run_id(id)?;
        let db = Connection::open_with_flags(path(root), OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Self::check_schema(&db)?;
        Self { db }.summary(id)
    }
    pub fn create_batch(
        &mut self,
        id: &str,
        snapshot: &Snapshot,
        settings: &Settings,
        concurrency: usize,
    ) -> Result<()> {
        snapshot.validate()?;
        settings.validate()?;
        let tx = self.db.transaction()?;
        tx.execute("INSERT INTO batches(batch_id,suite,manifest,input_fingerprint,snapshot_json,settings_json,concurrency,created_at_ms,state) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,'pending')",
            params![id,snapshot.suite,snapshot.manifest,snapshot.fingerprint()?,serde_json::to_string(snapshot)?,serde_json::to_string(settings)?,concurrency,now()])?;
        for (order, task) in snapshot.tasks.iter().enumerate() {
            let identity = TaskId::parse(&task.id)?;
            tx.execute(
                "INSERT INTO batch_tasks VALUES(?1,?2,?3,?4,?5,?6,'pending')",
                params![
                    id,
                    task.id,
                    order,
                    identity.group,
                    identity.variant,
                    snapshot.task_fingerprint(task)?
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn plan(&self, id: &str) -> Result<(Snapshot, Settings)> {
        let (snapshot,settings,expected):(String,String,String)=self.db.query_row("SELECT snapshot_json,settings_json,input_fingerprint FROM batches WHERE batch_id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).context("Batch does not exist")?;
        let snapshot: Snapshot = serde_json::from_str(&snapshot)?;
        ensure!(
            snapshot.fingerprint()? == expected,
            "Saved batch fingerprint does not match its inputs"
        );
        snapshot.validate()?;
        let settings: Settings = serde_json::from_str(&settings)?;
        settings.validate()?;
        Ok((snapshot, settings))
    }
    pub fn selected(&self, id: &str, retry: bool) -> Result<Vec<String>> {
        let mut stmt=self.db.prepare("SELECT task_id FROM batch_tasks WHERE batch_id=?1 AND (state='pending' OR (?2 AND state IN ('failed','failed_to_start','cancelled','interrupted'))) ORDER BY task_order")?;
        Ok(stmt
            .query_map(params![id, retry], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?)
    }
    pub fn begin(&self, id: &str) -> Result<()> {
        ensure!(self.db.execute("UPDATE batches SET state='running',started_at_ms=coalesce(started_at_ms,?2),finished_at_ms=NULL WHERE batch_id=?1",params![id,now()])? == 1,"Batch does not exist");
        Ok(())
    }
    pub fn reserve(
        &mut self,
        batch: &str,
        task: &str,
        id: &str,
        settings: &Settings,
    ) -> Result<()> {
        valid_run_id(id)?;
        let tx = self.db.transaction()?;
        let state: String = tx.query_row(
            "SELECT state FROM batch_tasks WHERE batch_id=?1 AND task_id=?2",
            params![batch, task],
            |r| r.get(0),
        )?;
        ensure!(
            state != "active" && state != "ready",
            "Task is active or already successful"
        );
        let number: i64 = tx.query_row(
            "SELECT coalesce(max(attempt_number),0)+1 FROM runs WHERE batch_id=?1 AND task_id=?2",
            params![batch, task],
            |r| r.get(0),
        )?;
        tx.execute("INSERT INTO runs(run_id,batch_id,task_id,attempt_number,state,reserved_at_ms,provider,model_requested,effort_requested,timeout_seconds) VALUES(?1,?2,?3,?4,'starting',?5,?6,?7,?8,?9)",params![id,batch,task,number,now(),settings.provider,settings.model,settings.effort,settings.timeout])?;
        tx.execute(
            "UPDATE batch_tasks SET state='active' WHERE batch_id=?1 AND task_id=?2",
            params![batch, task],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn complete(&mut self, c: &Completion) -> Result<()> {
        let json = serde_json::to_string(c)?;
        let tx = self.db.transaction()?;
        let (batch, task, previous): (String, String, Option<String>) = tx.query_row(
            "SELECT batch_id,task_id,completion_json FROM runs WHERE run_id=?1",
            [&c.run_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        ensure!(task == c.task, "Result task does not match its reservation");
        if let Some(previous) = previous {
            ensure!(
                previous == json,
                "Conflicting terminal evidence for run {}",
                c.run_id
            );
            return Ok(());
        }
        if let Some(r) = &c.report {
            let expected: (String, String, String, u64) = tx.query_row(
                "SELECT provider,model_requested,effort_requested,timeout_seconds FROM runs WHERE run_id=?1",
                [&c.run_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)))?;
            ensure!(
                expected
                    == (
                        r.provider.clone(),
                        r.model_requested.clone(),
                        r.effort_requested.clone(),
                        r.timeout_seconds
                    ),
                "Result configuration does not match its reservation"
            );
            ensure!(
                r.id == c.run_id && r.task == c.task && !r.state.active(),
                "Result identity or state is invalid"
            );
            ensure!(
                serde_json::to_value(r.state)? == c.state,
                "Result state is inconsistent"
            );
            for (index, attempt) in r.repair_attempts.iter().enumerate() {
                tx.execute(
                    "INSERT INTO repair_attempts VALUES(?1,?2,?3,?4,?5,?6,?7)",
                    params![
                        c.run_id,
                        index,
                        attempt.check,
                        attempt.passed,
                        fingerprint(&attempt.source)?,
                        serde_json::to_string(&attempt.commands)?,
                        attempt.error
                    ],
                )?;
            }
            let usage = r.usage.as_ref();
            let cost_basis = r
                .cost_basis
                .as_ref()
                .map(|basis| {
                    serde_json::to_value(basis).map(|v| v.as_str().unwrap_or_default().to_string())
                })
                .transpose()?;
            tx.execute("UPDATE runs SET created_at_ms=:created,finished_at_ms=:finished,provider_version=:provider_version,runner_version=:runner_version,runner_build=:runner_build,node_version=:node,vp_version=:vp,setup_started_at_ms=:setup_start,setup_finished_at_ms=:setup_finish,agent_started_at_ms=:agent_start,verification_started_at_ms=:verify_start,setup_seconds=:setup,agent_seconds=:agent,verification_seconds=:verify,elapsed_seconds=:elapsed,agent_exit_code=:agent_exit,verification_exit_code=:verify_exit,input_tokens=:input,cached_input_tokens=:cached,cache_write_input_tokens=:cache_write,output_tokens=:output,reasoning_output_tokens=:reasoning,cost_usd=:cost,cost_basis=:basis,cost_source=:cost_source,cost_models_json=:models,repair_check_count=:repairs,changed_file_count=:files,completed_turns=:turns,invalid_event_lines=:invalid,report_json=:report WHERE run_id=:id",
                named_params! {":id":c.run_id,":created":r.created_at_ms,":finished":r.finished_at_ms,":provider_version":r.provider_version,":runner_version":r.runner_version,":runner_build":option_env!("AGENT_UI_BUILD_ID"),":node":r.node_version,":vp":r.vp_version,":setup_start":r.setup_started_at_ms,":setup_finish":r.setup_finished_at_ms,":agent_start":r.agent_started_at_ms,":verify_start":r.verification_started_at_ms,
                    ":setup":r.setup_started_at_ms.map(|_|r.setup_seconds),":agent":r.agent_seconds,":verify":r.verification.as_ref().map(|v|v.seconds),":elapsed":r.finished_at_ms.and_then(|end|end.checked_sub(r.created_at_ms)).map(|ms|ms as f64/1000.0),":agent_exit":r.agent_exit_code,":verify_exit":r.verification.as_ref().and_then(|v|v.exit_code),
                    ":input":usage.map(|u|u.input_tokens),":cached":usage.map(|u|u.cached_input_tokens),":cache_write":usage.and_then(|u|u.cache_write_input_tokens),":output":usage.map(|u|u.output_tokens),":reasoning":usage.and_then(|u|u.reasoning_output_tokens),":cost":r.cost_usd,":basis":cost_basis,":cost_source":r.cost_source,":models":serde_json::to_string(&r.cost_models)?,":repairs":r.repair_attempts.len(),":files":(!r.after.is_empty()).then_some(r.changed_files.len()),":turns":r.completed_turns,":invalid":r.invalid_event_lines,":report":serde_json::to_string(r)? })?;
        } else {
            ensure!(
                matches!(c.state.as_str(), "failed_to_start" | "interrupted"),
                "Missing report for terminal result"
            );
        }
        crate::ledger_activity::seal(&tx, c)?;
        tx.execute("UPDATE runs SET state=?2,error=?3,recorded_at_ms=?4,recovered_at_ms=?5,completion_json=?6 WHERE run_id=?1",params![c.run_id,c.state,c.error,now(),c.recovered_at_ms,json])?;
        tx.execute(
            "UPDATE batch_tasks SET state=?3 WHERE batch_id=?1 AND task_id=?2",
            params![batch, task, c.state],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn summary(&self, id: &str) -> Result<BatchSummary> {
        let (suite, state): (String, String) = self
            .db
            .query_row(
                "SELECT suite,state FROM batches WHERE batch_id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .context("Batch does not exist")?;
        let mut stmt = self.db.prepare(
            "SELECT task_id,state FROM batch_tasks WHERE batch_id=?1 ORDER BY task_order",
        )?;
        let tasks = stmt
            .query_map([id], |r| {
                Ok(TaskStatus {
                    task: r.get(0)?,
                    state: r.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut task_states = BTreeMap::new();
        for task in &tasks {
            *task_states.entry(task.state.clone()).or_insert(0) += 1;
        }
        let (attempts, recorded) = self.db.query_row(
            "SELECT count(*),count(recorded_at_ms) FROM runs WHERE batch_id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        Ok(BatchSummary {
            batch_id: id.into(),
            suite,
            state,
            planned: tasks.len(),
            task_states,
            attempts,
            recorded,
            tasks,
        })
    }
    pub fn finish(&self, id: &str, recovered: bool) -> Result<()> {
        let summary = self.summary(id)?;
        if recovered && summary.state == "completed" && summary.successful() {
            return Ok(());
        }
        let state = if summary.successful() {
            "completed"
        } else if recovered {
            "interrupted"
        } else {
            "incomplete"
        };
        self.db.execute("UPDATE batches SET state=?2,finished_at_ms=?3,recovered_at_ms=coalesce(?4,recovered_at_ms) WHERE batch_id=?1",params![id,state,(!recovered).then(now),recovered.then(now)])?;
        Ok(())
    }
    pub fn deliver(&mut self, root: &Path, c: &Completion) -> Result<()> {
        let dir = root.join("ledger-pending");
        private_dir(&dir)?;
        let file = dir.join(format!("{}.json", c.run_id));
        valid_run_id(&c.run_id)?;
        write_json(&file, c)?;
        self.ingest_activity(root, &c.run_id, true)?;
        self.complete(c)?;
        crate::activity::remove(root, &c.run_id)?;
        fs::remove_file(file)?;
        fs::File::open(dir)?.sync_all()?;
        Ok(())
    }
    pub fn recover(store: &Store) -> Result<()> {
        if !path(store.root()).exists() {
            ensure!(
                !store.root().join("activity-pending").exists()
                    || fs::read_dir(store.root().join("activity-pending"))?
                        .next()
                        .is_none(),
                "Pending activity exists but the ledger is missing"
            );
            ensure!(
                !store.root().join("ledger-pending").exists()
                    || fs::read_dir(store.root().join("ledger-pending"))?
                        .next()
                        .is_none(),
                "Pending results exist but the ledger is missing"
            );
            return Ok(());
        }
        let mut ledger = Self::open(store.root(), false)?;
        let mut batches: std::collections::BTreeSet<String> = ledger.db.prepare(
            "SELECT batch_id FROM batches WHERE state='running' UNION SELECT batch_id FROM runs WHERE recorded_at_ms IS NULL")?
            .query_map([],|r|r.get(0))?.collect::<rusqlite::Result<_>>()?;
        let pending = store.root().join("ledger-pending");
        if pending.exists() {
            for entry in fs::read_dir(&pending)? {
                let file = entry?.path();
                if file.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                let c: Completion = serde_json::from_slice(&fs::read(&file)?)?;
                batches.insert(ledger.db.query_row(
                    "SELECT batch_id FROM runs WHERE run_id=?1",
                    [&c.run_id],
                    |r| r.get(0),
                )?);
                ledger.deliver(store.root(), &c)?;
            }
            fs::File::open(&pending)?.sync_all()?;
        }
        let unfinished: Vec<(String, String)> = ledger
            .db
            .prepare("SELECT run_id,task_id FROM runs WHERE recorded_at_ms IS NULL")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        for (id, task) in unfinished {
            let file = store.dir(&id)?.join("report.json");
            let c = if file.exists() {
                let mut report = store.load(&id)?;
                if report.state.active() {
                    report.interrupt();
                    store.save(&report)?;
                }
                Completion::report(report)?
            } else {
                Completion::no_report(
                    id,
                    task,
                    "Runner stopped before a report was saved".into(),
                    true,
                )
            };
            ledger.deliver(store.root(), &c)?;
        }
        for id in batches {
            ledger.finish(&id, true)?;
        }
        let activity = store.root().join("activity-pending");
        if activity.exists() {
            for entry in fs::read_dir(&activity)? {
                let file = entry?.path();
                if file.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                    continue;
                }
                let id = file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .context("Invalid activity journal name")?;
                let sealed: bool = ledger.db.query_row("SELECT EXISTS(SELECT 1 FROM run_capture WHERE run_id=?1 AND state<>'recording')",[id],|r|r.get(0))?;
                if sealed {
                    crate::activity::remove(store.root(), id)?;
                }
            }
        }
        Ok(())
    }
    pub fn ensure_removable(root: &Path, id: &str) -> Result<()> {
        ensure!(
            !root
                .join("ledger-pending")
                .join(format!("{id}.json"))
                .exists(),
            "Run {id} has pending ledger evidence. Restart to recover it"
        );
        if path(root).exists() {
            let db = Connection::open_with_flags(path(root), OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            db.busy_timeout(Duration::from_secs(2))?;
            Self::check_schema(&db)?;
            let pending: Option<bool> = db
                .query_row(
                    "SELECT recorded_at_ms IS NULL FROM runs WHERE run_id=?1",
                    [id],
                    |r| r.get(0),
                )
                .optional()?;
            ensure!(
                pending != Some(true),
                "Run {id} is not yet recorded. Restart to recover it"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::State;
    fn ledger() -> Ledger {
        Ledger::initialize(Connection::open_in_memory().unwrap(), true).unwrap()
    }
    fn batch(l: &mut Ledger) {
        l.create_batch(
            "batch",
            &crate::suite::tests::fixture(),
            &Settings::default(),
            4,
        )
        .unwrap();
    }
    #[test]
    fn retries_preserve_attempts_and_never_select_successful_tasks() {
        let mut l = ledger();
        batch(&mut l);
        l.reserve("batch", "a--baseline", "first", &Settings::default())
            .unwrap();
        let fail = Completion::no_report(
            "first".into(),
            "a--baseline".into(),
            "blocked".into(),
            false,
        );
        l.complete(&fail).unwrap();
        l.complete(&fail).unwrap();
        assert_eq!(l.summary("batch").unwrap().attempts, 1);
        assert_eq!(l.selected("batch", false).unwrap(), ["a--context"]);
        assert_eq!(
            l.selected("batch", true).unwrap(),
            ["a--baseline", "a--context"]
        );
        let mut conflicting = fail;
        conflicting.error = Some("different".into());
        assert!(l.complete(&conflicting).is_err());
        l.reserve("batch", "a--baseline", "second", &Settings::default())
            .unwrap();
        let mut report = Report::new(
            "second".into(),
            "a--baseline".into(),
            PathBuf::new(),
            &Settings::default(),
        );
        report.state = State::Ready;
        report.finished_at_ms = Some(report.created_at_ms + 1000);
        l.complete(&Completion::report(report).unwrap()).unwrap();
        assert_eq!(l.selected("batch", true).unwrap(), ["a--context"]);
        assert_eq!(l.summary("batch").unwrap().recorded, 2);
        assert!(
            l.reserve("batch", "a--baseline", "third", &Settings::default())
                .is_err()
        );
        assert!(
            l.db.execute("DELETE FROM runs WHERE run_id='first'", [])
                .is_err()
        );
    }
    #[test]
    fn absent_metrics_stay_null_and_recovery_has_no_false_finish() {
        let mut l = ledger();
        batch(&mut l);
        l.reserve("batch", "a--baseline", "run", &Settings::default())
            .unwrap();
        let mut report = Report::new(
            "run".into(),
            "a--baseline".into(),
            PathBuf::new(),
            &Settings::default(),
        );
        report.interrupt();
        l.complete(&Completion::report(report).unwrap()).unwrap();
        let valid: bool = l.db.query_row(
            "SELECT setup_seconds IS NULL AND input_tokens IS NULL AND finished_at_ms IS NULL AND elapsed_seconds IS NULL AND recovered_at_ms IS NOT NULL FROM runs",
            [], |row| row.get(0)).unwrap();
        assert!(valid);
    }
    #[test]
    fn schema_upgrade_keeps_existing_results_without_inventing_activity() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(include_str!("ledger.sql")).unwrap();
        db.execute("INSERT INTO batches VALUES('old','suite','manifest','hash','{}','{}',1,1000,NULL,NULL,NULL,'completed')",[]).unwrap();
        let ledger = Ledger::initialize(db, false).unwrap();
        assert_eq!(
            ledger
                .db
                .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            ledger
                .db
                .query_row(
                    "SELECT created_at_ms FROM batches WHERE batch_id='old'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            1000
        );
        assert_eq!(
            ledger
                .db
                .query_row("SELECT count(*) FROM run_capture", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn unsupported_database_is_not_modified() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("PRAGMA user_version=9;").unwrap();
        assert!(Ledger::initialize(db, true).is_err());
    }

    #[test]
    fn records_keep_exact_metrics_across_batches_and_failed_transactions() {
        let mut l = ledger();
        batch(&mut l);
        let settings = Settings::default();
        l.reserve("batch", "a--baseline", "measured", &settings)
            .unwrap();
        let mut r = Report::new(
            "measured".into(),
            "a--baseline".into(),
            PathBuf::new(),
            &settings,
        );
        r.created_at_ms = 1000;
        r.finished_at_ms = Some(5010);
        r.state = State::Failed;
        r.error = Some("Verification failed".into());
        r.setup_started_at_ms = Some(1001);
        r.setup_seconds = 0.4;
        r.agent_seconds = Some(2.1);
        r.verification = Some(crate::report::Check {
            exit_code: Some(1),
            seconds: 0.5,
        });
        r.usage = Some(crate::evidence::Usage {
            input_tokens: 1000,
            cached_input_tokens: 700,
            cache_write_input_tokens: Some(200),
            output_tokens: 25,
            reasoning_output_tokens: None,
        });
        r.cost_usd = Some(0.0123456);
        r.after.insert("file".into(), "hash".into());
        r.changed_files.push("file".into());
        r.repair_attempts.push(crate::repair::Attempt {
            check: "quality".into(),
            source: r.after.clone(),
            passed: false,
            commands: vec![],
            error: Some("Check failed".into()),
        });
        let c = Completion::report(r).unwrap();
        l.db.execute_batch("CREATE TRIGGER deny_result BEFORE UPDATE ON runs BEGIN SELECT RAISE(ABORT,'Disk failure fixture'); END;").unwrap();
        assert!(l.complete(&c).is_err());
        assert_eq!(l.summary("batch").unwrap().recorded, 0);
        assert_eq!(
            l.db.query_row("SELECT count(*) FROM repair_attempts", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        l.db.execute_batch("DROP TRIGGER deny_result;").unwrap();
        l.complete(&c).unwrap();
        l.complete(&c).unwrap();
        let row:(u64,u64,u64,u64,f64,f64)=l.db.query_row("SELECT input_tokens,cached_input_tokens,cache_write_input_tokens,output_tokens,cost_usd,elapsed_seconds FROM runs WHERE run_id='measured'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).unwrap();
        assert_eq!(row, (1000, 700, 200, 25, 0.0123456, 4.01));
        assert_eq!(
            l.db.query_row("SELECT count(*) FROM repair_attempts", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        l.create_batch(
            "second-batch",
            &crate::suite::tests::fixture(),
            &settings,
            4,
        )
        .unwrap();
        l.reserve("second-batch", "a--baseline", "next-model", &settings)
            .unwrap();
        l.complete(&Completion::no_report(
            "next-model".into(),
            "a--baseline".into(),
            "Preflight failed".into(),
            false,
        ))
        .unwrap();
        assert_eq!(l.summary("batch").unwrap().recorded, 1);
        assert_eq!(l.summary("second-batch").unwrap().recorded, 1);
        assert!(
            l.db.execute("UPDATE runs SET cost_usd=99 WHERE run_id='measured'", [])
                .is_err()
        );
    }

    #[test]
    fn unknown_task_or_wrong_configuration_cannot_commit_evidence() {
        let mut l = ledger();
        batch(&mut l);
        assert!(
            l.reserve("batch", "absent--task", "bad", &Settings::default())
                .is_err()
        );
        l.reserve("batch", "a--baseline", "run", &Settings::default())
            .unwrap();
        let mut r = Report::new(
            "run".into(),
            "a--baseline".into(),
            PathBuf::new(),
            &Settings::default(),
        );
        r.state = State::Failed;
        r.model_requested = "different-model".into();
        assert!(l.complete(&Completion::report(r).unwrap()).is_err());
        assert_eq!(l.summary("batch").unwrap().recorded, 0);
    }
}
