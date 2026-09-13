use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::time::{Duration, Instant};

pub(super) const PROGRESS_BOUND: Duration = Duration::from_secs(2);
const PARK_RELEASE_FAIL_SAFE: Duration = Duration::from_secs(6);

pub(super) struct WorkerPark {
    entered: SyncSender<()>,
    release: Receiver<()>,
}

pub(super) struct ParkControl {
    entered: Receiver<()>,
    release: Option<SyncSender<()>>,
}

pub(super) fn worker_park() -> (WorkerPark, ParkControl) {
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    (
        WorkerPark {
            entered: entered_tx,
            release: release_rx,
        },
        ParkControl {
            entered: entered_rx,
            release: Some(release_tx),
        },
    )
}

impl WorkerPark {
    pub(super) fn park(self, boundary: &str) {
        self.entered
            .send(())
            .unwrap_or_else(|_| panic!("{boundary} park controller disappeared"));
        self.release
            .recv_timeout(PARK_RELEASE_FAIL_SAFE)
            .unwrap_or_else(|error| panic!("{boundary} was not released within bound: {error}"));
    }
}

impl ParkControl {
    pub(super) fn wait_until_parked(&self, boundary: &str) {
        self.entered
            .recv_timeout(PROGRESS_BOUND)
            .unwrap_or_else(|error| panic!("{boundary} was not reached within bound: {error}"));
    }

    pub(super) fn release(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
    }
}

impl Drop for ParkControl {
    fn drop(&mut self) {
        self.release();
    }
}

pub(super) fn wait_until_progress(_description: &str, mut observed: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + PROGRESS_BOUND;
    while Instant::now() < deadline {
        if observed() {
            return true;
        }
        std::thread::yield_now();
    }
    observed()
}

pub(super) fn observe_within<T: Send + 'static>(
    mut observed: impl FnMut() -> Option<T> + Send + 'static,
) -> Result<T, RecvTimeoutError> {
    let (observed_tx, observed_rx) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let deadline = Instant::now() + PARK_RELEASE_FAIL_SAFE;
        while Instant::now() < deadline {
            if let Some(value) = observed() {
                let _ = observed_tx.send(value);
                return;
            }
            std::thread::yield_now();
        }
    });
    observed_rx.recv_timeout(PROGRESS_BOUND)
}
