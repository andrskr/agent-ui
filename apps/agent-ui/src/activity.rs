use crate::storage::{private_dir, valid_run_id};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

pub(crate) const CHUNK_BYTES: usize = 32 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Event {
    pub kind: String,
    pub phase: String,
    pub command_id: Option<String>,
    pub message_id: Option<String>,
    pub tool_call_id: Option<String>,
    pub parent_tool_call_id: Option<String>,
    pub provider_at_ms: Option<i64>,
    pub details: Value,
}
impl Event {
    pub fn new(kind: &str, phase: &str, details: Value) -> Self {
        Self {
            kind: kind.into(),
            phase: phase.into(),
            command_id: None,
            message_id: None,
            tool_call_id: None,
            parent_tool_call_id: None,
            provider_at_ms: None,
            details,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "record_type", rename_all = "snake_case")]
pub(crate) enum Record {
    Event(Event),
    Log {
        command_id: String,
        stream: String,
        offset: u64,
        bytes: Vec<u8>,
    },
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Entry {
    pub sequence: u64,
    pub observed_at_ms: u64,
    pub elapsed_ms: u64,
    #[serde(flatten)]
    pub record: Record,
}

#[derive(Clone, Default)]
pub(crate) struct Recorder(Option<Arc<Mutex<Sink>>>);
struct Sink {
    file: File,
    started: Instant,
    sequence: u64,
    failed: bool,
}
pub(crate) fn path(root: &Path, id: &str) -> Result<PathBuf> {
    valid_run_id(id)?;
    Ok(root.join("activity-pending").join(format!("{id}.jsonl")))
}
pub(crate) fn reserve(root: &Path, id: &str) -> Result<()> {
    let path = path(root, id)?;
    let dir = path.parent().context("Missing activity directory")?;
    private_dir(dir)?;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?
        .sync_all()?;
    File::open(dir)?.sync_all()?;
    Ok(())
}
pub(crate) fn remove(root: &Path, id: &str) -> Result<()> {
    let path = path(root, id)?;
    if path.exists() {
        fs::remove_file(&path)?;
        File::open(path.parent().context("Missing activity directory")?)?.sync_all()?;
    }
    Ok(())
}
impl Recorder {
    pub fn open(root: &Path, id: &str) -> Result<Self> {
        let path = path(root, id)?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let file = OpenOptions::new().append(true).open(path)?;
        ensure!(
            file.metadata()?.len() == 0,
            "Activity journal already has a writer"
        );
        Ok(Self(Some(Arc::new(Mutex::new(Sink {
            file,
            started: Instant::now(),
            sequence: 0,
            failed: false,
        })))))
    }
    pub fn enabled(&self) -> bool {
        self.0.is_some()
    }
    fn append(&self, record: Record) -> Result<()> {
        let Some(sink) = &self.0 else {
            return Ok(());
        };
        let mut sink = sink
            .lock()
            .map_err(|_| anyhow::anyhow!("Activity writer stopped"))?;
        ensure!(!sink.failed, "Activity journal write failed");
        let entry = Entry {
            sequence: sink.sequence + 1,
            observed_at_ms: crate::report::now(),
            elapsed_ms: sink.started.elapsed().as_millis() as u64,
            record,
        };
        let mut line = serde_json::to_vec(&entry)?;
        line.push(b'\n');
        if let Err(error) = sink
            .file
            .write_all(&line)
            .and_then(|()| sink.file.sync_data())
        {
            sink.failed = true;
            return Err(error).context("Cannot save run activity");
        }
        sink.sequence += 1;
        Ok(())
    }
    pub fn event(&self, event: Event) -> Result<()> {
        self.append(Record::Event(event))
    }
    pub fn note(&self, kind: &str, phase: &str, details: Value) -> Result<()> {
        self.event(Event::new(kind, phase, details))
    }
    pub fn log(&self, command_id: &str, stream: &str, offset: u64, bytes: &[u8]) -> Result<()> {
        if !self.enabled() {
            return Ok(());
        }
        for (index, chunk) in bytes.chunks(CHUNK_BYTES).enumerate() {
            self.append(Record::Log {
                command_id: command_id.into(),
                stream: stream.into(),
                offset: offset + (index * CHUNK_BYTES) as u64,
                bytes: chunk.to_vec(),
            })?;
        }
        Ok(())
    }
    pub fn finish(&self, state: &str) -> Result<()> {
        self.note("capture.finished", "run", json!({"state":state}))
    }
}

#[derive(Clone, Default)]
pub(crate) struct Trace {
    pub recorder: Recorder,
    pub phase: String,
    pub command_id: String,
}
impl Trace {
    pub fn new(recorder: Recorder, phase: &str) -> Self {
        Self {
            recorder,
            phase: phase.into(),
            command_id: uuid::Uuid::new_v4().to_string(),
        }
    }
    pub fn event(&self, kind: &str, command: &str, details: Value) -> Result<()> {
        let mut event = Event::new(kind, &self.phase, details);
        event.command_id = Some(command.into());
        self.recorder.event(event)
    }
}
