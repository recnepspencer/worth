use super::world::supply_chain::{certified_supply_chain_world, SupplyChainScale};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::inspection::RelationalMvccCostScope;
use worth_relational::facade::mvcc::{
    RelationalBranchTransactionAdmissionDenial, RelationalCancellationSource,
    RelationalInterruptionBoundary, RelationalOperationControl, RelationalOperationInterruption,
    RelationalPublicationOutcome, RelationalTransactionIntent,
};
use worth_relational::facade::publication::PatchStreamRequest;

#[test]
fn cancellation_before_observation_or_transaction_acquires_no_obligation() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = world.runtime.main_branch_identity();
    let source = RelationalCancellationSource::new();
    source.cancel();
    let control: RelationalOperationControl = source.token().into();
    let cost_scope = RelationalMvccCostScope::capture(&world.runtime, vec![identity.clone()]);

    assert_eq!(
        world
            .runtime
            .observe_branch_with_control(&identity, &control)
            .unwrap_err(),
        worth_relational::facade::branch::RelationalBranchBasisDenial::Cancelled,
    );
    let cancelled_observation_cost = world.runtime.observe_mvcc_counters(&cost_scope).unwrap();
    assert_eq!(
        cancelled_observation_cost
            .retention_cost_delta()
            .observation_acquires,
        0
    );
    assert_eq!(
        cancelled_observation_cost
            .retention_cost_delta()
            .observation_releases,
        0
    );
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    assert_eq!(
        world
            .runtime
            .begin_branch_transaction_with_control(
                &basis,
                RelationalTransactionIntent::ordinary(),
                control,
            )
            .unwrap_err(),
        RelationalBranchTransactionAdmissionDenial::Cancelled,
    );

    let cost = world.runtime.observe_mvcc_counters(&cost_scope).unwrap();
    assert_eq!(cost.retention_cost_delta().observation_acquires, 1);
    assert_eq!(cost.retention_cost_delta().transaction_acquires, 0);
    assert_eq!(
        cost.interruption_cost_delta().count(
            RelationalInterruptionBoundary::ObservationAdmission,
            RelationalOperationInterruption::Cancelled,
        ),
        1
    );
    assert_eq!(
        cost.interruption_cost_delta().count(
            RelationalInterruptionBoundary::TransactionAdmission,
            RelationalOperationInterruption::Cancelled,
        ),
        1
    );
}

#[test]
fn cancellation_during_preparation_releases_the_transaction_once() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = world.runtime.main_branch_identity();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let source = RelationalCancellationSource::new();
    let transaction = world
        .runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            source.token().into(),
        )
        .unwrap();
    let cost_scope = RelationalMvccCostScope::capture(&world.runtime, vec![identity]);
    source.cancel();

    let error = world
        .runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    let worth_relational::facade::transactions::TransactionCommitError::Interrupted {
        interruption,
        ..
    } = error
    else {
        panic!("preparation cancellation remains typed");
    };
    assert_eq!(
        interruption.interruption(),
        RelationalOperationInterruption::Cancelled
    );
    assert_eq!(
        interruption.boundary(),
        RelationalInterruptionBoundary::ProposalValidation
    );
    let cost = world.runtime.observe_mvcc_counters(&cost_scope).unwrap();
    assert_eq!(cost.retention_cost_delta().transaction_releases, 1);
    assert_eq!(cost.retention_cost_delta().candidate_acquires, 0);
    assert_eq!(
        cost.interruption_cost_delta().count(
            RelationalInterruptionBoundary::ProposalValidation,
            RelationalOperationInterruption::Cancelled,
        ),
        1
    );
}

#[test]
fn cancellation_after_candidate_creation_wins_before_branch_movement() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = world.runtime.main_branch_identity();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let source = RelationalCancellationSource::new();
    let transaction = world
        .runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            source.token().into(),
        )
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let before_reference = world
        .runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let commit_count_before = world.runtime.history().immutable_commit_count();
    let stream_before = world
        .runtime
        .publication()
        .read_patch_stream(PatchStreamRequest::default())
        .unwrap();
    let retention_before = world.runtime.retention().inspect_plan();
    let cost_scope = RelationalMvccCostScope::capture(&world.runtime, vec![identity]);
    source.cancel();

    assert!(matches!(
        world
            .runtime
            .publication_port()
            .compare_and_publish(candidate),
        RelationalPublicationOutcome::Interrupted(_)
    ));
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before_reference,
    );
    assert_eq!(
        world.runtime.history().immutable_commit_count(),
        commit_count_before
    );
    assert_eq!(
        world
            .runtime
            .publication()
            .read_patch_stream(PatchStreamRequest::default())
            .unwrap(),
        stream_before
    );
    assert_eq!(world.runtime.retention().inspect_plan(), retention_before);
    let cost = world.runtime.observe_mvcc_counters(&cost_scope).unwrap();
    assert_eq!(cost.retention_cost_delta().candidate_releases, 1);
    assert_eq!(
        cost.interruption_cost_delta().count(
            RelationalInterruptionBoundary::PublicationPreflight,
            RelationalOperationInterruption::Cancelled,
        ),
        1
    );
}

#[test]
fn cancellation_after_snapshot_pin_preserves_and_releases_the_exact_observation() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = world.runtime.main_branch_identity();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let source = RelationalCancellationSource::new();
    source.cancel();

    assert!(matches!(
        world.runtime.begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            source.token().into(),
        ),
        Err(RelationalBranchTransactionAdmissionDenial::Cancelled)
    ));
    assert_eq!(
        world
            .runtime
            .read_truth()
            .inspect_snapshot(&snapshot)
            .unwrap()
            .version_id,
        basis.observation().version_id()
    );
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
}

#[test]
fn cancellation_after_linearization_preserves_performed_commit_and_settles_once() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = world.runtime.main_branch_identity();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let source = RelationalCancellationSource::new();
    let transaction = world
        .runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            source.token().into(),
        )
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let before = world
        .runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let performed = match world
        .runtime
        .publication_port()
        .compare_and_publish(candidate)
    {
        RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("uncancelled Supply Chain publication performs: {outcome:?}"),
    };
    assert_ne!(
        world
            .runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before
    );

    source.cancel();
    let committed = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    let interruption = committed
        .late_interruption()
        .expect("post-linearization cancellation remains attached to performed evidence");
    assert_eq!(
        interruption.interruption(),
        RelationalOperationInterruption::Cancelled
    );
    assert_eq!(
        interruption.boundary(),
        RelationalInterruptionBoundary::Settlement
    );
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
}

#[test]
fn elapsed_deadline_is_distinct_from_cancellation() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let control = RelationalOperationControl::uninterrupted()
        .with_deadline(std::time::Instant::now() - std::time::Duration::from_millis(1));
    assert_eq!(
        world
            .runtime
            .observe_branch_with_control(&world.runtime.main_branch_identity(), &control)
            .unwrap_err(),
        worth_relational::facade::branch::RelationalBranchBasisDenial::TimedOut,
    );
}
