pub(crate) use std::sync::mpsc::sync_channel;
pub(crate) use {std::sync::Arc, std::sync::Barrier};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use crate::relational_certification::ordinary_context::owner_service_completion::world::committed_fork;
pub(crate) use worth_relational::facade::transactions::WorkerIntentBatch;
pub(crate) use {
    worth_relational::facade::mvcc::RelationalOperationControl,
    worth_relational::facade::mvcc::RelationalPublicationOutcome,
    worth_relational::facade::mvcc::RelationalTransactionIntent,
};

pub(crate) const LOCALITY_TIMEOUT: Duration = std::time::Duration::from_secs(2);

pub(crate) fn assert_pause_reached(reached: Arc<Barrier>, release: Arc<Barrier>) {
    let (arrived_tx, arrived_rx) = sync_channel(1);
    std::thread::spawn(move || {
        reached.wait();
        let _ = arrived_tx.send(());
    });
    if arrived_rx.recv_timeout(LOCALITY_TIMEOUT).is_err() {
        std::thread::spawn(move || {
            release.wait();
        });
        panic!("publication never reached its real branch critical-section pause");
    }
}

pub(crate) fn wait_for_coordination_wait(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    identity: &worth_relational::facade::branch::RelationalBranchIdentity,
    waits_before: u64,
    timeout: Duration,
) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if coordination_waits(runtime, identity) > waits_before {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::yield_now();
    }
}

pub(crate) fn coordination_waits(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    identity: &worth_relational::facade::branch::RelationalBranchIdentity,
) -> u64 {
    runtime
        .observe_branch_sharing(std::slice::from_ref(identity))
        .expect("the owner-issued branch remains publicly inspectable")
        .coordination_waits()
}
