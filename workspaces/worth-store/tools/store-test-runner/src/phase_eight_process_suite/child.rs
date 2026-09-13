use std::process::{Command, ExitStatus};
use std::time::{Duration, Instant};

pub(super) fn run_within(command: &mut Command, timeout: Duration) -> Result<ExitStatus, String> {
    let mut child = command
        .spawn()
        .map_err(|error| format!("launch Phase 8 process suite: {error}"))?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(None) => {
                return Err(terminate(
                    &mut child,
                    format!("Phase 8 process suite exceeded {timeout:?}"),
                ));
            }
            Err(error) => {
                return Err(terminate(
                    &mut child,
                    format!("inspect Phase 8 process suite child: {error}"),
                ));
            }
        }
    }
}

fn terminate(child: &mut std::process::Child, primary: String) -> String {
    let kill = child
        .kill()
        .err()
        .map(|error| format!("kill Phase 8 process suite child: {error}"));
    let wait = child
        .wait()
        .err()
        .map(|error| format!("reap Phase 8 process suite child: {error}"));
    [Some(primary), kill, wait]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("; ")
}
