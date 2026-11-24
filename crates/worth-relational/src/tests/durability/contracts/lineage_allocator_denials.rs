use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use worth_relational_contracts::internal_port::lineage::data::artifacts::published_artifact::PublishedLineageArtifactFixturePort as _;
use worth_relational_contracts::internal_port::lineage::data::events::LineageEventRecordFieldsPort as _;

#[test]
fn tail_created_lineage_allocator_exhaustion_denies_before_replay() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "lineage-overflow-checkpoint");
    medium.file_embedded_checkpoint(execution, &runtime);
    create_entity_outcome(execution, &runtime, "lineage-overflow-tail");
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    plan.tail_log[0]
        .readmitted_envelope_mut_for_test(execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut_for_test()[0]
        .owner_targets_mut()[0] = crate::identity::data::LineageId(u64::MAX - 1);
    plan.tail_log[0]
        .readmitted_envelope_mut_for_test(execution)
        .rebuild_lineage_basis_for_test();

    let recovered_medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&recovered_medium);
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("tail lineage allocator exhaustion must deny");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("tail lineage id exhausted"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn live_lineage_allocator_exhaustion_denies_before_public_effects() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let baseline = create_entity_outcome(execution, &runtime, "lineage-exhaustion-baseline");
    let baseline_entities = runtime
        .storage_access()
        .storage_stats(execution)
        .unwrap()
        .live_entities;
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
        .commit(execution, &runtime)
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
        runtime
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .live_entities,
        baseline_entities
    );
}
