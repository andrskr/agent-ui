use crate::{process::OwnedChild, storage::Store};
use anyhow::{Result, ensure};
use std::{
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

enum PreviewPhase {
    Starting(Instant),
    Ready,
}
pub struct Preview {
    id: String,
    url: String,
    process: OwnedChild,
    _lock: File,
    phase: PreviewPhase,
}
impl Preview {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn is_ready(&self) -> bool {
        matches!(self.phase, PreviewPhase::Ready)
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    fn is_running(&mut self) -> Result<bool> {
        Ok(self.process.try_wait()?.is_none())
    }
    pub fn start(store: &Store, id: &str, vp: &Path) -> Result<Self> {
        let lock = store.preview_lock(id)?;
        let report = store.load(id)?;
        ensure!(
            !report.state.active(),
            "Wait for the run to finish before opening a preview"
        );
        let files = store.files(id)?;
        let dir = files.root();
        let app = files.app();
        ensure!(
            app.join("node_modules").is_dir(),
            "This run has no installed dependencies"
        );
        let socket = TcpListener::bind("127.0.0.1:0")?;
        let port = socket.local_addr()?.port();
        drop(socket);
        let mut command = Command::new(vp);
        command
            .current_dir(app)
            .args([
                "dev",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--strictPort",
            ])
            .stdin(Stdio::null())
            .stdout(File::create(dir.join("preview.log"))?)
            .stderr(File::create(dir.join("preview.stderr.log"))?);
        let process = OwnedChild::spawn(&mut command)?;
        Ok(Self {
            id: id.into(),
            url: format!("http://127.0.0.1:{port}/"),
            process,
            _lock: lock,
            phase: PreviewPhase::Starting(Instant::now()),
        })
    }
    fn ready(&mut self) -> Result<bool> {
        ensure!(
            self.process.try_wait()?.is_none(),
            "Preview stopped. See preview.stderr.log"
        );
        let address = self.url.trim_start_matches("http://").trim_end_matches('/');
        if let Ok(mut stream) =
            TcpStream::connect_timeout(&address.parse()?, Duration::from_millis(50))
        {
            stream.set_read_timeout(Some(Duration::from_millis(50)))?;
            stream.write_all(b"GET / HTTP/1.0\r\nHost: localhost\r\n\r\n")?;
            let mut buffer = [0; 80];
            if let Ok(n) = stream.read(&mut buffer) {
                return Ok(String::from_utf8_lossy(&buffer[..n])
                    .split_whitespace()
                    .nth(1)
                    == Some("200"));
            }
        }
        Ok(false)
    }
    /// Returns true once when the server is ready to open.
    pub fn poll(&mut self) -> Result<bool> {
        match self.phase {
            PreviewPhase::Starting(started) => {
                if self.ready()? {
                    self.phase = PreviewPhase::Ready;
                    return Ok(true);
                }
                ensure!(
                    started.elapsed() < Duration::from_secs(30),
                    "Preview did not start within 30 seconds"
                );
            }
            PreviewPhase::Ready => ensure!(
                self.is_running()?,
                "Preview stopped. See preview.stderr.log"
            ),
        }
        Ok(false)
    }
}
