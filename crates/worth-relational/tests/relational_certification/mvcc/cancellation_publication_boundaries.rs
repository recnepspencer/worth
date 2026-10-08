use std::sync::{Arc, Barrier};

use super::world::supply_chain::{certified_supply_chain_world, SupplyChainScale};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::{
    RelationalCancellationSource, RelationalInterruptionBoundary, RelationalOperationControl,
    RelationalOperationInterruption, RelationalPublicationOutcome, RelationalTransactionIntent,
};

#[test]
fn cancellation_immediately_before_the_supply_chain_critical_section_moves_nothing() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = world.runtime.main_branch_identity();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let before = world
        .runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let control = RelationalOperationControl::uninterrupted().with_injected_interruption(
        RelationalInterruptionBoundary::BeforeCriticalSection,
        RelationalOperationInterruption::Cancelled,
        1,
    );
    let transaction = world
        .runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let event = match world
        .runtime
        .publication_port()
        .compare_and_publish(candidate)
    {
        RelationalPublicationOutcome::Interrupted(event) => event,
        outcome => panic!("pre-critical cancellation must win: {outcome:?}"),
    };
    assert_eq!(
        event.boundary(),
        RelationalInterruptionBoundary::BeforeCriticalSection
    );
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before
    );
}

#[test]
fn cancellation_inside_the_supply_chain_critical_section_is_deferred() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = world.runtime.main_branch_identity();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let before = world
        .runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let source = RelationalCancellationSource::new();
    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let control = RelationalOperationControl::from(source.token())
        .with_critical_section_pause(Arc::clone(&reached), Arc::clone(&release));
    let transaction = world
        .runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let port = world.runtime.publication_port();
    let publisher = std::thread::spawn(move || port.compare_and_publish(candidate));

    reached.wait();
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before,
        "the certification pause is inside coordination but before movement"
    );
    source.cancel();
    release.wait();

    let performed = match publisher.join().unwrap() {
        RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("inside-critical cancellation must perform: {outcome:?}"),
    };
    assert_ne!(
        world
            .runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before
    );
    let interruption = performed
        .late_interruption()
        .expect("deferred cancellation remains attached to performed evidence");
    assert_eq!(
        interruption.boundary(),
        RelationalInterruptionBoundary::AfterLinearization
    );
    let committed = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
}
