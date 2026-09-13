use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use super::world::committed_fork;
use worth_relational::facade::mvcc::{
    RelationalOperationControl, RelationalPublicationOutcome, RelationalTransactionIntent,
};
use worth_relational::facade::transactions::WorkerIntentBatch;

const LOCALITY_TIMEOUT: Duration = Duration::from_secs(2);

#[test]
fn parked_branch_lifecycle_call_does_not_block_unrelated_basis_service() {
    let (runtime, services, parked_identity) = committed_fork("parked-lifecycle");
    let parked_basis = services
        .basis_port()
        .admit_branch_basis(&parked_identity)
        .expect("the parked branch basis is owner-admitted");
    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let control = RelationalOperationControl::uninterrupted()
        .with_critical_section_pause(Arc::clone(&reached), Arc::clone(&release));
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            &parked_basis,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .expect("the exact parked basis opens a controlled transaction");
    transaction
        .push_batch(WorkerIntentBatch::new("park-lifecycle-coordination"))
        .expect("the locality batch remains within its declared budget");
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(transaction)
        .expect("the controlled transaction prepares canonically");

    let publication_port = services.publication_port();
    let publication_worker =
        std::thread::spawn(move || publication_port.compare_and_publish(candidate));
    assert_pause_reached(Arc::clone(&reached), Arc::clone(&release));

    let waits_before = coordination_waits(&runtime, &parked_identity);
    let lifecycle_port = services.lifecycle_port();
    let lifecycle_identity = parked_identity.clone();
    let lifecycle_worker =
        std::thread::spawn(move || lifecycle_port.archive_branch(&lifecycle_identity));
    let lifecycle_is_parked =
        wait_for_coordination_wait(&runtime, &parked_identity, waits_before, LOCALITY_TIMEOUT);

    let main_identity = runtime.main_branch_identity();
    let basis_port = services.basis_port();
    let (completed_tx, completed_rx) = sync_channel(1);
    let basis_worker = std::thread::spawn(move || {
        let result = basis_port
            .observe_branch(&main_identity)
            .map(|(descriptor, _basis)| descriptor.branch_id().clone());
        let _ = completed_tx.send(result);
    });
    let unrelated_result = completed_rx.recv_timeout(LOCALITY_TIMEOUT);
    let lifecycle_stayed_parked = !lifecycle_worker.is_finished();

    release.wait();
    let publication = publication_worker
        .join()
        .expect("the parked publication worker does not panic");
    let archived = lifecycle_worker
        .join()
        .expect("the parked lifecycle worker does not panic");
    basis_worker
        .join()
        .expect("the unrelated basis worker does not panic");

    assert!(
        lifecycle_is_parked,
        "the lifecycle call never contended on its exact branch coordination cell"
    );
    assert!(
        lifecycle_stayed_parked,
        "the branch lifecycle call completed before its target cell was released"
    );
    assert!(
        matches!(unrelated_result, Ok(Ok(ref branch)) if branch.0 == "main"),
        "unrelated basis work must finish while lifecycle remains parked: {unrelated_result:?}"
    );
    archived.expect("archive completes after the exact branch cell is released");
    let performed = match publication {
        RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("parked canonical publication did not perform: {outcome:?}"),
    };
    services
        .settlement_port()
        .settle_performed_publication(performed)
        .expect("the performed locality publication settles canonically");
}

fn assert_pause_reached(reached: Arc<Barrier>, release: Arc<Barrier>) {
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

fn wait_for_coordination_wait(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    identity: &worth_relational::facade::branch::RelationalBranchIdentity,
    waits_before: u64,
    timeout: Duration,
) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if coordination_waits(runtime, identity) > waits_before {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::yield_now();
    }
}

fn coordination_waits(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    identity: &worth_relational::facade::branch::RelationalBranchIdentity,
) -> u64 {
    runtime
        .observe_branch_sharing(std::slice::from_ref(identity))
        .expect("the owner-issued branch remains publicly inspectable")
        .coordination_waits()
}
