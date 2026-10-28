//! Bound setup by completed work and report the last stalled writer phase.

use std::path::Path;
use std::time::{Duration, Instant};

use super::child_lifecycle::ProcessChildGuard;

pub(super) fn wait(
    child: &mut ProcessChildGuard,
    start: &Path,
    operation_count: usize,
) -> Result<(), String> {
    let started = Instant::now();
    // Setup contains every submitted durable operation, two intermediate
    // checkpoints, and a final checkpoint. An aggregate 120 s startup timer
    // mistook a progressing 96-operation history for a hung process under load.
    // Keep a 60 s inactivity watchdog plus an explicit five seconds/operation
    // and 120 s fixed-work ceiling. These are liveness guards, not performance
    // assertions; neither changes the fixture or the observed recovery proof.
    let ceiling = Duration::from_secs(120 + operation_count as u64 * 5);
    let mut last_progress = started;
    let mut phase = String::from("initialization");
    let ready = start.with_extension("ready");
    let progress = start.with_extension("progress");
    while !ready.exists() {
        if let Ok(next) = std::fs::read_to_string(&progress)
            && !next.is_empty()
            && next != phase
        {
            phase = next;
            last_progress = Instant::now();
        }
        if started.elapsed() >= ceiling || last_progress.elapsed() >= Duration::from_secs(60) {
            return Err(format!(
                "C8 writer setup stalled: phase={phase}, elapsed={:?}, inactive={:?}, operations={operation_count}, marker={}",
                started.elapsed(), last_progress.elapsed(), ready.display()
            ));
        }
        if let Some(status) = child
            .child_mut()
            .try_wait()
            .map_err(|error| format!("poll C8 writer setup: {error}"))?
        {
            return Err(format!("C8 writer exited during {phase}: {status}"));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    eprintln!(
        "C8 writer ready: operations={operation_count}, elapsed={:?}, last_phase={phase}",
        started.elapsed()
    );
    Ok(())
}
