use crate::{
    activity::{self, Entry, Record},
    ledger::{Completion, Ledger},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{OptionalExtension, Transaction, params};
use serde_json::{Value, json};
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::Path,
};

impl Ledger {
    pub(crate) fn enable_capture(&self, root: &Path, id: &str) -> Result<()> {
        self.db.execute(
            "INSERT INTO run_capture(run_id,capture_version,state) VALUES(?1,1,'recording')",
            [id],
        )?;
        activity::reserve(root, id)
    }
    /// The coordinator is the only SQLite writer. Worker journals can still be growing.
    pub(crate) fn ingest_activity(
        &mut self,
        root: &Path,
        id: &str,
        final_read: bool,
    ) -> Result<()> {
        let capture: Option<(String, u64)> = self
            .db
            .query_row(
                "SELECT state,journal_offset FROM run_capture WHERE run_id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((state, offset)) = capture else {
            return Ok(());
        };
        if state != "recording" {
            return Ok(());
        }
        let path = activity::path(root, id)?;
        if !path.exists() {
            if final_read {
                self.db.execute(
                    "UPDATE run_capture SET error='Activity journal is missing' WHERE run_id=?1",
                    [id],
                )?;
            }
            return Ok(());
        }
        let mut reader = BufReader::new(File::open(path)?);
        reader.seek(SeekFrom::Start(offset))?;
        self.ingest_activity_reader(id, reader, offset, final_read)
    }
    fn ingest_activity_reader(
        &mut self,
        id: &str,
        mut reader: impl BufRead,
        mut offset: u64,
        final_read: bool,
    ) -> Result<()> {
        loop {
            let tx = self.db.transaction()?;
            let mut count = 0;
            let mut ended = false;
            while count < 128 {
                let mut line = Vec::new();
                let bytes = reader.read_until(b'\n', &mut line)?;
                if bytes == 0 {
                    ended = true;
                    break;
                }
                if line.last() != Some(&b'\n') {
                    if final_read {
                        tx.execute("UPDATE run_capture SET error='Incomplete final activity record' WHERE run_id=?1",[id])?;
                    }
                    ended = true;
                    break;
                }
                let entry: Entry =
                    serde_json::from_slice(&line).context("Invalid activity journal record")?;
                offset += bytes as u64;
                insert(&tx, id, &entry, offset)?;
                count += 1;
            }
            tx.commit()?;
            if ended || !final_read {
                break;
            }
        }
        Ok(())
    }
    pub(crate) fn capture_error(&self, id: &str) -> Result<Option<String>> {
        Ok(self
            .db
            .query_row(
                "SELECT error FROM run_capture WHERE run_id=?1 AND state='partial'",
                [id],
                |r| r.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten())
    }
    pub(crate) fn read_events(root: &Path, id: &str, after: u64, limit: usize) -> Result<Value> {
        crate::storage::valid_run_id(id)?;
        ensure!(
            (1..=1000).contains(&limit),
            "Event limit must be between 1 and 1000"
        );
        let db = rusqlite::Connection::open_with_flags(
            crate::ledger::path(root),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        Self::check_schema(&db)?;
        let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        ensure!(
            db.query_row(
                "SELECT EXISTS(SELECT 1 FROM runs WHERE run_id=?1)",
                [id],
                |r| r.get::<_, bool>(0)
            )?,
            "Run does not exist"
        );
        if version == 1 {
            return Ok(
                json!({"run_id":id,"capture":{"state":"not_recorded"},"events":[],"next_after":null}),
            );
        }
        let capture: Option<Value> = db.query_row("SELECT state,capture_version,last_sequence,log_bytes,error FROM run_capture WHERE run_id=?1",[id],|r|Ok(json!({"state":r.get::<_,String>(0)?,"version":r.get::<_,u64>(1)?,"last_sequence":r.get::<_,u64>(2)?,"log_bytes":r.get::<_,u64>(3)?,"error":r.get::<_,Option<String>>(4)?}))).optional()?;
        let rows = db.prepare("SELECT sequence,observed_at_ms,elapsed_ms,kind,phase,command_id,message_id,tool_call_id,parent_tool_call_id,provider_at_ms,details_json FROM run_events WHERE run_id=?1 AND sequence>?2 ORDER BY sequence LIMIT ?3")?
            .query_map(params![id,after,limit+1], |r| Ok(json!({
                "sequence":r.get::<_,u64>(0)?,"observed_at_ms":r.get::<_,u64>(1)?,"elapsed_ms":r.get::<_,u64>(2)?,"kind":r.get::<_,String>(3)?,"phase":r.get::<_,String>(4)?,"command_id":r.get::<_,Option<String>>(5)?,"message_id":r.get::<_,Option<String>>(6)?,"tool_call_id":r.get::<_,Option<String>>(7)?,"parent_tool_call_id":r.get::<_,Option<String>>(8)?,"provider_at_ms":r.get::<_,Option<i64>>(9)?,"details_json":r.get::<_,String>(10)?
            })))?.collect::<rusqlite::Result<Vec<_>>>()?;
        let has_more = rows.len() > limit;
        let events = rows
            .into_iter()
            .take(limit)
            .map(|mut row| -> Result<Value> {
                let raw = row
                    .as_object_mut()
                    .context("Invalid event")?
                    .remove("details_json")
                    .context("Missing details")?;
                row["details"] = serde_json::from_str(raw.as_str().context("Invalid details")?)?;
                Ok(row)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(
            json!({"run_id":id,"capture":capture.unwrap_or(json!({"state":"not_recorded"})),"next_after":if has_more { events.last().map(|e|e["sequence"].clone()) } else { None },"events":events}),
        )
    }
}

fn insert(tx: &Transaction<'_>, id: &str, entry: &Entry, offset: u64) -> Result<()> {
    let (last, old_offset, state): (u64, u64, String) = tx.query_row(
        "SELECT last_sequence,journal_offset,state FROM run_capture WHERE run_id=?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    ensure!(state == "recording", "Activity capture is sealed");
    ensure!(
        entry.sequence == last + 1 && offset > old_offset,
        "Activity sequence or offset is inconsistent"
    );
    let mut bytes = 0;
    match &entry.record {
        Record::Event(e) => {
            tx.execute(
                "INSERT INTO run_events VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                params![
                    id,
                    entry.sequence,
                    entry.observed_at_ms,
                    entry.elapsed_ms,
                    e.kind,
                    e.phase,
                    e.command_id,
                    e.message_id,
                    e.tool_call_id,
                    e.parent_tool_call_id,
                    e.provider_at_ms,
                    serde_json::to_string(&e.details)?
                ],
            )?;
        }
        Record::Log {
            command_id,
            stream,
            offset,
            bytes: content,
        } => {
            ensure!(
                !content.is_empty() && content.len() <= activity::CHUNK_BYTES,
                "Invalid activity chunk size"
            );
            let expected:u64=tx.query_row("SELECT coalesce(max(byte_offset+length(content)),0) FROM run_log_chunks WHERE run_id=?1 AND command_id=?2 AND stream=?3",params![id,command_id,stream],|r|r.get(0))?;
            ensure!(
                *offset == expected,
                "Activity output contains a gap or duplicate"
            );
            tx.execute(
                "INSERT INTO run_log_chunks VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    id,
                    entry.sequence,
                    entry.observed_at_ms,
                    entry.elapsed_ms,
                    command_id,
                    stream,
                    offset,
                    content
                ],
            )?;
            bytes = content.len();
        }
    }
    tx.execute("UPDATE run_capture SET last_sequence=?2,journal_offset=?3,log_bytes=log_bytes+?4 WHERE run_id=?1",params![id,entry.sequence,offset,bytes])?;
    Ok(())
}

pub(crate) fn seal(tx: &Transaction<'_>, c: &Completion) -> Result<()> {
    let capture: Option<(u64, Option<String>)> = tx
        .query_row(
            "SELECT last_sequence,error FROM run_capture WHERE run_id=?1 AND state='recording'",
            [&c.run_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((sequence, error)) = capture else {
        return Ok(());
    };
    let finished: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM run_events WHERE run_id=?1 AND sequence=?2 AND kind='capture.finished')",params![c.run_id,sequence],|r|r.get(0))?;
    let incomplete: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM run_events WHERE run_id=?1 AND kind='capture.incomplete')",
        [&c.run_id],
        |r| r.get(0),
    )?;
    let state = if !incomplete && error.is_none() && finished && c.state != "interrupted" {
        "complete"
    } else if sequence == 0 && c.state == "failed_to_start" && error.is_none() {
        "not_started"
    } else {
        "partial"
    };
    let error = error.or_else(|| {
        (state == "partial").then(|| "Capture ended without a complete final record".to_string())
    });
    tx.execute(
        "UPDATE run_capture SET state=?2,error=?3 WHERE run_id=?1",
        params![c.run_id, state, error],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{activity::Event, settings::Settings};
    use std::io::Cursor;
    fn fixture() -> Ledger {
        let mut ledger = Ledger::temporary().unwrap();
        ledger
            .create_batch(
                "batch",
                &crate::suite::tests::fixture(),
                &Settings::default(),
                4,
            )
            .unwrap();
        ledger
            .reserve("batch", "a--baseline", "run", &Settings::default())
            .unwrap();
        ledger
            .db
            .execute(
                "INSERT INTO run_capture(run_id,capture_version,state) VALUES('run',1,'recording')",
                [],
            )
            .unwrap();
        ledger
    }
    fn event(sequence: u64, kind: &str, details: Value) -> Entry {
        Entry {
            sequence,
            observed_at_ms: 1000 + sequence,
            elapsed_ms: sequence,
            record: Record::Event(Event::new(kind, "agent", details)),
        }
    }
    fn encoded(entries: &[Entry]) -> Vec<u8> {
        entries
            .iter()
            .flat_map(|entry| {
                let mut bytes = serde_json::to_vec(entry).unwrap();
                bytes.push(b'\n');
                bytes
            })
            .collect()
    }
    fn read(ledger: &mut Ledger, bytes: &[u8], final_read: bool) -> Result<()> {
        let offset: u64 = ledger.db.query_row(
            "SELECT journal_offset FROM run_capture WHERE run_id='run'",
            [],
            |r| r.get(0),
        )?;
        let mut reader = Cursor::new(bytes);
        reader.set_position(offset);
        ledger.ingest_activity_reader("run", reader, offset, final_read)
    }
    #[test]
    fn replay_and_split_tail_preserve_exact_bytes_once() {
        let mut ledger = fixture();
        let first = encoded(&[event(1, "run.started", json!({}))]);
        let log = Entry {
            sequence: 2,
            observed_at_ms: 1002,
            elapsed_ms: 2,
            record: Record::Log {
                command_id: "cmd".into(),
                stream: "stderr".into(),
                offset: 0,
                bytes: vec![0, 255, 10, 195, 169],
            },
        };
        let second = encoded(&[log]);
        let mut partial = first.clone();
        partial.extend_from_slice(&second[..second.len() / 2]);
        read(&mut ledger, &partial, false).unwrap();
        assert_eq!(
            ledger
                .db
                .query_row("SELECT last_sequence FROM run_capture", [], |r| r
                    .get::<_, u64>(0))
                .unwrap(),
            1
        );
        let mut all = first;
        all.extend_from_slice(&second);
        read(&mut ledger, &all, true).unwrap();
        read(&mut ledger, &all, true).unwrap();
        let row:(u64,u64,Vec<u8>)=ledger.db.query_row("SELECT c.last_sequence,c.log_bytes,l.content FROM run_capture c JOIN run_log_chunks l USING(run_id)",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        assert_eq!(row, (2, 5, vec![0, 255, 10, 195, 169]));
    }
    #[test]
    fn failed_sql_write_rolls_back_rows_and_cursor_for_replay() {
        let mut ledger = fixture();
        ledger.db.execute_batch("CREATE TRIGGER fail_log BEFORE INSERT ON run_log_chunks BEGIN SELECT RAISE(ABORT,'Disk failure fixture'); END;").unwrap();
        let bytes = encoded(&[
            event(1, "run.started", json!({})),
            Entry {
                sequence: 2,
                observed_at_ms: 1002,
                elapsed_ms: 2,
                record: Record::Log {
                    command_id: "cmd".into(),
                    stream: "stdout".into(),
                    offset: 0,
                    bytes: b"hello".to_vec(),
                },
            },
        ]);
        assert!(read(&mut ledger, &bytes, true).is_err());
        assert_eq!(
            ledger
                .db
                .query_row("SELECT count(*) FROM run_events", [], |r| r
                    .get::<_, u64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            ledger
                .db
                .query_row("SELECT journal_offset FROM run_capture", [], |r| r
                    .get::<_, u64>(0))
                .unwrap(),
            0
        );
        ledger.db.execute_batch("DROP TRIGGER fail_log").unwrap();
        read(&mut ledger, &bytes, true).unwrap();
        assert_eq!(
            ledger
                .db
                .query_row("SELECT log_bytes FROM run_capture", [], |r| r
                    .get::<_, u64>(0))
                .unwrap(),
            5
        );
    }
    #[test]
    fn incomplete_capture_is_distinct_from_the_run_result() {
        for (entries, expected) in [
            (vec![event(1, "run.started", json!({}))], "partial"),
            (vec![event(1, "capture.finished", json!({}))], "complete"),
            (
                vec![
                    event(1, "capture.incomplete", json!({})),
                    event(2, "capture.finished", json!({})),
                ],
                "partial",
            ),
        ] {
            let mut ledger = fixture();
            read(&mut ledger, &encoded(&entries), true).unwrap();
            let c = Completion::no_report(
                "run".into(),
                "a--baseline".into(),
                "start failure".into(),
                false,
            );
            ledger.complete(&c).unwrap();
            ledger.complete(&c).unwrap();
            let state: String = ledger
                .db
                .query_row("SELECT state FROM run_capture", [], |r| r.get(0))
                .unwrap();
            assert_eq!(state, expected);
            assert!(ledger.db.execute("DELETE FROM run_events", []).is_err());
            assert!(
                ledger
                    .db
                    .execute("UPDATE run_capture SET state='recording'", [])
                    .is_err()
            );
            assert!(ledger.db.execute("INSERT INTO run_events VALUES('run',99,0,0,'late','agent',NULL,NULL,NULL,NULL,NULL,'{}')",[]).is_err());
        }
    }
    #[test]
    fn usage_view_uses_latest_message_snapshot_and_keeps_missing_values_null() {
        let mut ledger = fixture();
        let mut entries = Vec::new();
        for (sequence, id, tokens) in [(1, "m1", 10), (2, "m1", 12), (3, "m2", 7)] {
            let mut e = Event::new(
                "message.usage",
                "agent",
                json!({"request_id":"req","model":"fixture","usage":{"input_tokens":tokens,"output_tokens":2}}),
            );
            e.message_id = Some(id.into());
            entries.push(Entry {
                sequence,
                observed_at_ms: 1000 + sequence,
                elapsed_ms: sequence,
                record: Record::Event(e),
            });
        }
        entries.push(event(
            4,
            "message.usage",
            json!({"usage":{"input_tokens":999}}),
        ));
        read(&mut ledger, &encoded(&entries), true).unwrap();
        let result:(u64,u64,u64)=ledger.db.query_row("SELECT count(*),sum(uncached_input_tokens),count(cached_input_tokens) FROM run_request_usage",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        assert_eq!(result, (2, 19, 0));
    }
    #[test]
    fn torn_tail_and_sequence_gaps_cannot_be_marked_complete() {
        let mut ledger = fixture();
        let mut bytes = encoded(&[event(1, "capture.finished", json!({}))]);
        bytes.extend_from_slice(b"{partial");
        read(&mut ledger, &bytes, true).unwrap();
        ledger
            .complete(&Completion::no_report(
                "run".into(),
                "a--baseline".into(),
                "failed".into(),
                false,
            ))
            .unwrap();
        assert_eq!(
            ledger
                .db
                .query_row("SELECT state FROM run_capture", [], |r| r
                    .get::<_, String>(0))
                .unwrap(),
            "partial"
        );
        let mut ledger = fixture();
        assert!(
            read(
                &mut ledger,
                &encoded(&[event(2, "capture.finished", json!({}))]),
                true
            )
            .is_err()
        );
    }
}
