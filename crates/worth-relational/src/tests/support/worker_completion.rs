//! Preserve a worker's panic and bound completion through its final drops.
use std::sync::mpsc::{sync_channel, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::Duration;

pub(crate) fn join_completed_worker<T: Send + 'static>(
    completion: Result<(), RecvTimeoutError>,
    worker: JoinHandle<T>,
    bound: Duration,
    timeout_message: &str,
) -> T {
    if matches!(completion, Err(RecvTimeoutError::Timeout)) {
        panic!("{timeout_message}");
    }
    // Completion notification can precede the worker's final Drop. A join
    // proxy keeps that tail bounded too and carries the original panic.
    let (finished, joined) = sync_channel(1);
    std::thread::spawn(move || {
        let _ = finished.send(worker.join());
    });
    match joined.recv_timeout(bound) {
        Ok(Ok(value)) => value,
        Ok(Err(worker_panic)) => std::panic::resume_unwind(worker_panic),
        Err(RecvTimeoutError::Timeout) => panic!("{timeout_message}"),
        Err(RecvTimeoutError::Disconnected) => panic!("worker join proxy disconnected"),
    }
}
