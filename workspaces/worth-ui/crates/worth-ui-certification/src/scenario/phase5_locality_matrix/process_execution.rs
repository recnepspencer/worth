use std::io::Read;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(super) const DEADLINE_ENV: &str = "WORTH_UI_PHASE5_MATRIX_DEADLINE_MS";
pub(super) const CANCEL_ENV: &str = "WORTH_UI_PHASE5_MATRIX_CANCEL_FILE";
const MAXIMUM_EXECUTION: Duration = Duration::from_secs(8 * 60 + 30);

pub(super) struct CapturedOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
}

pub(super) fn new_deadline() -> Result<u128, String> {
    now_millis().map(|now| now + MAXIMUM_EXECUTION.as_millis())
}

pub(super) fn deadline_from_environment() -> Result<u128, String> {
    match std::env::var(DEADLINE_ENV) {
        Ok(value) => value
            .parse()
            .map_err(|_| "matrix deadline is not an integer".to_owned()),
        Err(std::env::VarError::NotPresent) => new_deadline(),
        Err(denial) => Err(format!("matrix deadline environment: {denial}")),
    }
}

pub(super) fn run_until(
    command: &mut Command,
    deadline: u128,
    label: &str,
) -> Result<CapturedOutput, String> {
    run_until_with_cancellation(command, deadline, label, None)
}

#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-spawn=1: Certification drives independent actors to expose concurrency behavior."
)]
pub(super) fn run_until_with_cancellation(
    command: &mut Command,
    deadline: u128,
    label: &str,
    cancellation: Option<&AtomicBool>,
) -> Result<CapturedOutput, String> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|denial| format!("matrix process {label}: {denial}"))?;
    let Some(mut stdout) = child.stdout.take() else {
        stop(&mut child);
        return Err(format!("matrix process {label} omitted stdout"));
    };
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(denial) => {
                stop(&mut child);
                let _ = reader.join();
                return Err(format!("matrix process {label} status: {denial}"));
            }
        }
        if cancellation_requested(cancellation) {
            stop(&mut child);
            let _ = reader.join();
            return Err(format!("matrix process {label} was cancelled"));
        }
        let now = match now_millis() {
            Ok(now) => now,
            Err(denial) => {
                stop(&mut child);
                let _ = reader.join();
                return Err(denial);
            }
        };
        if now >= deadline {
            stop(&mut child);
            let _ = reader.join();
            return Err(format!(
                "matrix process {label} exceeded the eight-and-a-half-minute deadline"
            ));
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    let stdout = reader
        .join()
        .map_err(|_| format!("matrix process {label} output reader panicked"))?
        .map_err(|denial| format!("matrix process {label} output: {denial}"))?;
    Ok(CapturedOutput { status, stdout })
}

fn cancellation_requested(cancellation: Option<&AtomicBool>) -> bool {
    cancellation.is_some_and(|flag| flag.load(Ordering::Acquire))
        || std::env::var_os(CANCEL_ENV).is_some_and(|path| std::path::Path::new(&path).exists())
}

fn stop(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn now_millis() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .map_err(|denial| format!("matrix wall-clock deadline: {denial}"))
}

impl CapturedOutput {
    pub(super) fn status(&self) -> ExitStatus {
        self.status
    }

    pub(super) fn stdout(&self) -> &[u8] {
        &self.stdout
    }
}
