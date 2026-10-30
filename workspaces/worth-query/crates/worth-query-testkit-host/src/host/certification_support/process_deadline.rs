//! Capture a child's streams and bound both execution and termination.
pub use std::io::{self, Read};
pub use std::process::{Child, Command, ExitStatus, Output, Stdio};
pub use std::sync::{mpsc, Arc, Mutex};
pub use std::time::{Duration, Instant};

pub use worth_harness::tooling::process_deadline::DEFAULT_CHILD_PROCESS_HANG_GUARD;

pub fn output(command: &mut Command) -> io::Result<Output> {
    output_with_deadline(command, DEFAULT_CHILD_PROCESS_HANG_GUARD)
}

pub fn output_with_deadline(command: &mut Command, bound: Duration) -> io::Result<Output> {
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut running = RunningChild {
        child: Some(child),
        bound,
    };
    let child = running.child.as_mut().unwrap();
    let (finished, completion) = mpsc::channel();
    let stdout = capture(child.stdout.take().unwrap(), finished.clone());
    let stderr = capture(child.stderr.take().unwrap(), finished);
    let started = Instant::now();
    let deadline = started + bound;
    let mut streams = 2;
    let mut status = None;
    loop {
        if status.is_none() {
            status = running.child.as_mut().unwrap().try_wait()?;
        }
        if let (Some(status), 0) = (status, streams) {
            drop(running.child.take()); // try_wait has already reaped the process.
            return Ok(Output {
                status,
                stdout: stdout.lock().unwrap().clone(),
                stderr: stderr.lock().unwrap().clone(),
            });
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            let mut child = running.child.take().unwrap();
            let killed = child.kill();
            let cleanup_deadline = Instant::now() + bound;
            let reaped = wait_for_exit(&mut child, cleanup_deadline);
            // Reaping can precede EOF delivery. Retain the final captured bytes
            // when readers finish, without giving cleanup an unbounded wait.
            while streams != 0 {
                let remaining = cleanup_deadline.saturating_duration_since(Instant::now());
                match completion.recv_timeout(remaining) {
                    Ok(_) => streams -= 1,
                    Err(_) => break,
                }
            }
            panic!(
                "{}",
                expiry_report(
                    bound,
                    started.elapsed(),
                    &killed,
                    &reaped,
                    &stdout.lock().unwrap(),
                    &stderr.lock().unwrap(),
                )
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
            // EOF can precede the OS exit status, or a child can close its
            // pipes and hang. The execution deadline governs both cases.
            std::thread::yield_now();
        }
    }
}

pub struct RunningChild {
    pub child: Option<Child>,
    pub bound: Duration,
}

impl Drop for RunningChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let started = Instant::now();
            let killed = child.kill();
            let reaped = wait_for_exit(&mut child, started + self.bound);
            if killed.is_err() || !matches!(reaped, Ok(Some(_))) {
                eprintln!(
                    "child cleanup elapsed={:?}; kill={killed:?}; reap={reaped:?}",
                    started.elapsed()
                );
            }
        }
    }
}

// None means the same bound expired during reaping, even if kill failed.
pub fn wait_for_exit(child: &mut Child, deadline: Instant) -> io::Result<Option<ExitStatus>> {
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        std::thread::yield_now();
    }
}

pub fn expiry_report(
    bound: Duration,
    elapsed: Duration,
    killed: &io::Result<()>,
    reaped: &io::Result<Option<ExitStatus>>,
    stdout: &[u8],
    stderr: &[u8],
) -> String {
    format!(
        "child exceeded {bound:?}; elapsed={elapsed:?}; kill={killed:?}; reap={reaped:?} (None means reap deadline expired)\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(stdout), String::from_utf8_lossy(stderr),
    )
}

#[expect(
    clippy::disallowed_methods,
    reason = "stream-reader thread-spawn=1: Testkit child-stream readers prevent pipe backpressure while the bounded process owner retains termination custody."
)]
pub fn capture(
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
