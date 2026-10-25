//! A wedged cell is killed and reaped; its captured diagnostics name the row.
use std::io::Read;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

pub(super) fn output(
    command: &mut Command,
    subject: &str,
    backstop: Duration,
) -> Result<Output, String> {
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("{subject}: spawn: {error}"))?;
    let mut child = RunningCell(child);
    // Drain both pipes while the child runs, so full diagnostics cannot block it.
    let stdout = capture(child.0.stdout.take().unwrap());
    let stderr = capture(child.0.stderr.take().unwrap());
    let status = wait(&mut child.0, backstop);
    if status.is_err() {
        // A successful kill is followed by wait: no orphan or zombie cell.
        child
            .0
            .kill()
            .map_err(|error| format!("{subject}: kill after {status:?}: {error}"))?;
        child
            .0
            .wait()
            .map_err(|error| format!("{subject}: reap: {error}"))?;
    }
    let stdout = stdout
        .join()
        .map_err(|_| format!("{subject}: stdout reader panicked"))?
        .map_err(|error| format!("{subject}: stdout: {error}"))?;
    let stderr = stderr
        .join()
        .map_err(|_| format!("{subject}: stderr reader panicked"))?
        .map_err(|error| format!("{subject}: stderr: {error}"))?;
    let status = status.map_err(|error| {
        format!(
            "{subject}: {error}\n{}{}",
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&stderr)
        )
    })?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}
struct RunningCell(Child);
impl Drop for RunningCell {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

fn capture(
    mut pipe: impl Read + Send + 'static,
) -> std::thread::JoinHandle<std::io::Result<Vec<u8>>> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}
fn wait(child: &mut Child, backstop: Duration) -> Result<std::process::ExitStatus, String> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|error| format!("wait: {error}"))? {
            return Ok(status);
        }
        if started.elapsed() >= backstop {
            return Err(format!("child-process backstop expired after {backstop:?}"));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

const PARKED_CHILD: &str = "WORTH_MATRIX_PARKED_CHILD";
const CHILD_TEST: &str =
    "checkpoint_recovery::parallel_history::matrix::table::cell_process::cell_backstop_child";
#[test]
fn cell_backstop_child() {
    if std::env::var_os(PARKED_CHILD).is_some() {
        println!("matrix child deliberately parked before member 40 arrives");
        use std::io::Write;
        std::io::stdout().flush().unwrap();
        loop {
            std::thread::park();
        }
    }
}
#[test]
fn child_backstop_kills_and_reaps_a_cell_that_never_finishes() {
    let mut child = Command::new(std::env::current_exe().unwrap());
    child
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env(PARKED_CHILD, "1");
    let stop = output(
        &mut child,
        "missing member 40 injection",
        Duration::from_secs(5),
    )
    .expect_err("a deliberately parked child must fail within the backstop");
    assert!(
        stop.contains("child-process backstop expired")
            && stop.contains("deliberately parked before member 40 arrives"),
        "{stop}"
    );
}
