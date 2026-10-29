pub(crate) use {std::sync::mpsc, std::sync::mpsc::sync_channel, std::sync::mpsc::TryRecvError};
pub(crate) use {std::sync::Arc, std::sync::Barrier};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use {crate::relational_certification::ordinary_context::owner_service_completion::world::empty_runtime};
pub(crate) use {worth_relational::facade::branch::RelationalOwnerLifecycleObservation};
pub(crate) use {worth_relational::facade::mvcc::RelationalOperationControl, worth_relational::facade::mvcc::RelationalPublicationOutcome, worth_relational::facade::mvcc::RelationalTransactionIntent};
pub(crate) use {worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) const LIFECYCLE_BOUND: Duration = std::time::Duration::from_secs(2);
pub(crate) const OWNER_CLOSE_START_FAIL_SAFE: Duration = std::time::Duration::from_secs(6);

pub(crate) fn release_controlled_pause(
    release: Arc<Barrier>,
) -> Result<(), std::sync::mpsc::RecvTimeoutError> {
    let (released_tx, released_rx) = sync_channel(1);
    std::thread::spawn(move || {
        release.wait();
        let _ = released_tx.send(());
    });
    released_rx.recv_timeout(LIFECYCLE_BOUND)
}

pub(crate) fn assert_pause_reached(reached: Arc<Barrier>, release: Arc<Barrier>) {
    let (arrived_tx, arrived_rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        reached.wait();
        let _ = arrived_tx.send(());
    });
    if arrived_rx.recv_timeout(LIFECYCLE_BOUND).is_err() {
        std::thread::spawn(move || {
            release.wait();
        });
        panic!("publication never reached its controlled critical section");
    }
}

pub(crate) fn wait_until(timeout: Duration, mut observed: impl FnMut() -> bool) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if observed() {
            return true;
        }
        std::thread::yield_now();
    }
    observed()
}