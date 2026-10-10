//! Captured child output and termination under one assertion deadline.
use std::io::{self, Read};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

const CHILD_PROCESS_DEADLINE: Duration = Duration::from_secs(30);

pub(crate) fn output(command: &mut Command) -> io::Result<Output> {
    let mut running = RunningChild(
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?,
    );
    let (finished, completion) = mpsc::channel();
    let stdout = capture(running.0.stdout.take().unwrap(), finished.clone());
    let stderr = capture(running.0.stderr.take().unwrap(), finished);
    let deadline = Instant::now() + CHILD_PROCESS_DEADLINE;
    let mut streams = 2;
    let mut status = None;
    loop {
        if status.is_none() {
            status = running.0.try_wait()?;
        }
        if let (Some(status), 0) = (status, streams) {
            return Ok(Output {
                status,
                stdout: stdout.lock().unwrap().clone(),
                stderr: stderr.lock().unwrap().clone(),
            });
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            let _ = running.0.kill();
            let reaped = running.0.wait();
            panic!(
                "child exceeded {CHILD_PROCESS_DEADLINE:?}; reap={reaped:?}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&stdout.lock().unwrap()),
                String::from_utf8_lossy(&stderr.lock().unwrap()),
            );
        }
        if streams != 0 {
            match completion.recv_timeout(remaining) {
                Ok(result) => {
                    streams -= 1;
                    result?;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(io::Error::other("child output reader disconnected"));
                }
            }
        } else {
            // EOF can precede the OS exit status. Yield only during that race;
            // a child that closes both streams and hangs still has a deadline.
            std::thread::yield_now();
        }
    }
}

struct RunningChild(Child);
impl Drop for RunningChild {
    fn drop(&mut self) {
        // Every early I/O error or panic also terminates and reaps the child.
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn capture(
    mut pipe: impl Read + Send + 'static,
    finished: mpsc::Sender<io::Result<()>>,
) -> Arc<Mutex<Vec<u8>>> {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&bytes);
    std::thread::spawn(move || {
        let result = (|| {
            let mut chunk = [0; 4096];
            loop {
                let read = pipe.read(&mut chunk)?;
                if read == 0 {
                    return Ok(());
                }
                captured.lock().unwrap().extend_from_slice(&chunk[..read]);
            }
        })();
        let _ = finished.send(result);
    });
    bytes
}
