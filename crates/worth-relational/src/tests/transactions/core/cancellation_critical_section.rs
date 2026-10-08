use std::sync::{Arc, Barrier};

use crate::facade::history::BranchId;
use crate::facade::mvcc::{
    RelationalCancellationSource, RelationalInterruptionBoundary, RelationalOperationControl,
    RelationalOperationInterruption, RelationalTransactionIntent,
};
use crate::tests::support::*;

#[test]
fn cancellation_while_waiting_for_publication_coordination_denies_before_movement() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "contended-cancellation-anchor");
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let source = RelationalCancellationSource::new();
    let control = RelationalOperationControl::from(source.token());
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .unwrap();
    transaction
        .push_batch(
            batch_create("contended-cancellation-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let publication_cell = candidate.publication_cell_for_test();
    let coordination = Arc::clone(publication_cell.coordination());
    let held_coordination = coordination.enter();
    let before = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let port = runtime.publication_port();

    let publisher = std::thread::spawn(move || port.compare_and_publish(candidate));
    let wait_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while coordination.wait_count() == 0 && std::time::Instant::now() < wait_deadline {
        std::thread::yield_now();
    }
    assert_eq!(coordination.wait_count(), 1);
    source.cancel();
    drop(held_coordination);

    let event = match publisher.join().unwrap() {
        crate::mvcc::RelationalPublicationOutcome::Interrupted(event) => event,
        outcome => panic!("waited cancellation must deny before movement: {outcome:?}"),
    };
    assert_eq!(
        event.interruption(),
        RelationalOperationInterruption::Cancelled
    );
    assert_eq!(
        event.boundary(),
        RelationalInterruptionBoundary::BeforeCriticalSection
    );
    assert_eq!(
        runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before
    );
}

#[test]
fn cancellation_inside_publication_critical_section_defers_until_performed() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "critical-cancellation-anchor");
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let source = RelationalCancellationSource::new();
    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let control = RelationalOperationControl::from(source.token())
        .with_critical_section_pause(Arc::clone(&reached), Arc::clone(&release));
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .unwrap();
    transaction
        .push_batch(
            batch_create("critical-cancellation-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let before = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let port = runtime.publication_port();

    let publisher = std::thread::spawn(move || port.compare_and_publish(candidate));
    reached.wait();
    assert_eq!(
        runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before,
        "the pause is inside coordination but before linearization"
    );
    source.cancel();
    release.wait();

    let performed = match publisher.join().unwrap() {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("inside-critical cancellation must preserve movement: {outcome:?}"),
    };
    assert_ne!(
        runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before
    );
    let interruption = performed
        .late_interruption()
        .expect("deferred cancellation is attached after linearization");
    assert_eq!(
        interruption.interruption(),
        RelationalOperationInterruption::Cancelled
    );
    assert_eq!(
        interruption.boundary(),
        RelationalInterruptionBoundary::AfterLinearization
    );
    let committed = runtime.settle_performed_publication(performed).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
}
