use anyhow::{Context, Result, ensure};
use nix::{
    sys::signal::{Signal, killpg},
    unistd::Pid,
};
use std::{
    fs::File,
    io::{Read, Write},
    os::unix::process::CommandExt,
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Default)]
pub struct Cancel(Arc<AtomicBool>);
impl Cancel {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
    pub fn install_signal_handler(&self) -> Result<()> {
        let signal = self.clone();
        ctrlc::set_handler(move || signal.cancel()).context("Cannot install the stop handler")
    }
}
pub struct OwnedChild {
    child: Child,
    stopped: bool,
}
impl OwnedChild {
    pub fn spawn(command: &mut Command) -> Result<Self> {
        Ok(Self {
            child: command
                .process_group(0)
                .spawn()
                .context("Cannot start process")?,
            stopped: false,
        })
    }
    pub fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }
    pub fn stop(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        let _ = killpg(Pid::from_raw(self.child.id() as i32), Signal::SIGTERM);
        let deadline = Instant::now() + Duration::from_millis(500);
        while Instant::now() < deadline {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        let _ = killpg(Pid::from_raw(self.child.id() as i32), Signal::SIGKILL);
        let _ = self.child.wait();
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Termination {
    Exited,
    TimedOut,
    Cancelled,
}

#[derive(Debug)]
pub struct Outcome {
    pub code: Option<i32>,
    pub seconds: f64,
    pub termination: Termination,
}
pub fn execute(
    command: &mut Command,
    log: &Path,
    stderr: &Path,
    input: Option<&str>,
    timeout: Duration,
    cancel: &Cancel,
    mut on_tick: impl FnMut(&[Vec<u8>], f64) -> Result<()>,
) -> Result<Outcome> {
    ensure!(
        !cancel.is_cancelled(),
        "Run cancelled before starting the process"
    );
    let started = Instant::now();
    command
        .stdout(File::create(log)?)
        .stderr(File::create(stderr)?);
    command.stdin(if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    let mut process = OwnedChild::spawn(command)?;
    let writer = if let Some(input) = input {
        let mut stdin = process
            .child
            .stdin
            .take()
            .context("Process input pipe is missing")?;
        let input = input.as_bytes().to_vec();
        Some(thread::spawn(move || stdin.write_all(&input)))
    } else {
        None
    };
    let mut running = RunningCommand { process, writer };
    let mut reader = File::open(log)?;
    let mut output = LogLines::default();
    loop {
        let lines = output.read(&mut reader)?;
        on_tick(&lines, started.elapsed().as_secs_f64())?;
        let cancelled = cancel.is_cancelled();
        let timed_out = started.elapsed() >= timeout;
        if cancelled || timed_out {
            running.process.stop();
        }
        if let Some(status) = running.process.try_wait()? {
            running.process.stop();
            if let Some(writer) = running.writer.take() {
                let written = writer
                    .join()
                    .map_err(|_| anyhow::anyhow!("Prompt writer stopped"))?;
                if status.success() && !cancelled && !timed_out {
                    written.context("Cannot submit the complete prompt")?;
                }
            }
            let mut lines = output.read(&mut reader)?;
            lines.extend(output.finish());
            on_tick(&lines, started.elapsed().as_secs_f64())?;
            return Ok(Outcome {
                code: status.code(),
                seconds: started.elapsed().as_secs_f64(),
                termination: if cancelled {
                    Termination::Cancelled
                } else if timed_out {
                    Termination::TimedOut
                } else {
                    Termination::Exited
                },
            });
        }
        thread::sleep(Duration::from_millis(100));
    }
}

#[derive(Default)]
struct LogLines {
    pending: Vec<u8>,
}

impl LogLines {
    fn read(&mut self, reader: &mut impl Read) -> std::io::Result<Vec<Vec<u8>>> {
        reader.read_to_end(&mut self.pending)?;
        let Some(last) = self.pending.iter().rposition(|byte| *byte == b'\n') else {
            return Ok(Vec::new());
        };
        let remaining = self.pending.split_off(last + 1);
        let complete = std::mem::replace(&mut self.pending, remaining);
        Ok(complete
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(<[u8]>::to_vec)
            .collect())
    }

    fn finish(self) -> Option<Vec<u8>> {
        (!self.pending.is_empty()).then_some(self.pending)
    }
}
pub fn checked(result: &Outcome, phase: &str) -> Result<()> {
    ensure!(
        result.termination != Termination::Cancelled,
        "Run cancelled during {phase}"
    );
    ensure!(
        result.termination != Termination::TimedOut,
        "Time limit reached during {phase}"
    );
    ensure!(
        result.code == Some(0),
        "{phase} failed (exit {:?}). See its log",
        result.code
    );
    Ok(())
}

struct RunningCommand {
    process: OwnedChild,
    writer: Option<thread::JoinHandle<std::io::Result<()>>>,
}
impl Drop for RunningCommand {
    fn drop(&mut self) {
        // Stop the reader before joining a writer that can be blocked on its input pipe.
        self.process.stop();
        if let Some(writer) = self.writer.take() {
            let _ = writer.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LogLines, Outcome, Termination, checked};

    #[test]
    fn log_records_survive_every_chunk_size_including_split_utf8() {
        let bytes = "{\"text\":\"café\"}\n\n{\"type\":\"turn.completed\"}\nlast".as_bytes();
        for size in 1..=bytes.len() {
            let mut decoder = LogLines::default();
            let mut records = Vec::new();
            for mut chunk in bytes.chunks(size) {
                records.extend(decoder.read(&mut chunk).unwrap());
            }
            records.extend(decoder.finish());
            assert_eq!(
                records,
                [
                    "{\"text\":\"café\"}".as_bytes(),
                    b"{\"type\":\"turn.completed\"}",
                    b"last",
                ],
                "chunk size {size}"
            );
        }
    }

    #[test]
    fn partial_records_wait_for_a_newline_or_process_exit() {
        let mut decoder = LogLines::default();
        assert!(decoder.read(&mut &b"partial"[..]).unwrap().is_empty());
        assert!(decoder.read(&mut &b""[..]).unwrap().is_empty());
        assert_eq!(
            decoder.read(&mut &b" record\nnext"[..]).unwrap(),
            [b"partial record"]
        );
        assert_eq!(decoder.finish().as_deref(), Some(&b"next"[..]));
    }

    #[test]
    fn complete_records_are_not_replayed_on_empty_polls_or_exit() {
        let mut decoder = LogLines::default();
        assert_eq!(
            decoder.read(&mut &b"first\nsecond\n"[..]).unwrap(),
            [b"first".as_slice(), b"second"]
        );
        assert!(decoder.read(&mut &b""[..]).unwrap().is_empty());
        assert!(decoder.finish().is_none());
    }

    #[test]
    fn only_a_normal_zero_exit_passes_a_process_phase() {
        for termination in [
            Termination::Exited,
            Termination::TimedOut,
            Termination::Cancelled,
        ] {
            for code in [None, Some(0), Some(7)] {
                let outcome = Outcome {
                    code,
                    seconds: 0.0,
                    termination,
                };
                let result = checked(&outcome, "Verification");
                match (termination, code) {
                    (Termination::Exited, Some(0)) => assert!(result.is_ok()),
                    _ => assert!(result.is_err(), "{outcome:?}"),
                }
            }
        }
    }

    #[test]
    fn cancellation_and_timeout_keep_their_reason_even_with_a_zero_exit() {
        let cancelled = Outcome {
            code: Some(0),
            seconds: 1.0,
            termination: Termination::Cancelled,
        };
        assert_eq!(
            checked(&cancelled, "Codex").unwrap_err().to_string(),
            "Run cancelled during Codex"
        );
        let timeout = Outcome {
            termination: Termination::TimedOut,
            ..cancelled
        };
        assert_eq!(
            checked(&timeout, "Codex").unwrap_err().to_string(),
            "Time limit reached during Codex"
        );
    }
}
