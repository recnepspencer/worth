use super::*;

#[test]
fn tail_created_lineage_allocator_exhaustion_denies_before_replay() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "lineage-overflow-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity_outcome(&runtime, "lineage-overflow-tail");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.tail_log[0]
        .envelope_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .targets[0] = crate::identity::data::LineageId(u64::MAX - 1);
    plan.tail_log[0]
        .envelope_mut_for_test()
        .rebuild_lineage_basis_for_test();

    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(plan)
        .expect_err("tail lineage allocator exhaustion must deny");

    assert_eq!(error.class, RecoveryFailureClass::CorruptSegment);
    assert!(error.detail.contains("tail lineage id exhausted"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn live_lineage_allocator_exhaustion_denies_before_public_effects() {
    let runtime = persisted_runtime_with_test_schema();
    let baseline = create_entity_outcome(&runtime, "lineage-exhaustion-baseline");
    let baseline_entities = runtime.storage_access().storage_stats().live_entities;
    let (_, event_frontier) = runtime.lineage.identity_frontiers();
    runtime
        .lineage
        .set_identity_frontiers(u64::MAX - 1, event_frontier);

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("lineage-exhaustion-denied"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let error = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect_err("lineage reservation must deny at the allocator boundary");

    assert!(format!("{error:?}").contains("lineage id allocator exhausted"));
    assert_eq!(
        runtime.lineage.identity_frontiers(),
        (u64::MAX - 1, event_frontier)
    );
    assert_eq!(
        runtime
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .expect("baseline head"),
        baseline.commit
    );
    assert_eq!(
        runtime.storage_access().storage_stats().live_entities,
        baseline_entities
    );
}
