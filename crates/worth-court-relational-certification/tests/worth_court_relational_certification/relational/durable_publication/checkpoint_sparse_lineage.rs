use super::branch_history_publications::create_entity_outcome_on_branch;
use super::branch_history_publications::fork;
use worth_relational::facade::durability::RecoveryVerificationMode as RecoveryMode;
use worth_relational::facade::history::BranchId;
use worth_testkit_relational::aspect_storage::publications::changed_entities;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::create_entity_outcome;
use worth_testkit_relational::durable_publication::durable_runtime;

#[test]
fn branch_local_sparse_slot_create_publishes_and_recovers_exact_lineage() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "sparse-slot-shared-root");
    fork(execution, &runtime, "sparse-slot-feature");
    let feature = BranchId("sparse-slot-feature".into());
    create_entity_outcome(execution, &runtime, "sparse-slot-main-divergence");
    let feature_create = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "sparse-slot-feature-create",
        feature.clone(),
    );
    let feature_entity = changed_entities(&feature_create)[0];
    assert!(feature_entity.local_slot.0 > 1);
    let identity = runtime.branch_identity(&feature).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let lineage_id = runtime
        .lineage_node_at_observation(execution, &basis.observation(), feature_entity)
        .unwrap()
        .unwrap()
        .node()
        .lineage_id();
    let event_targets = runtime
        .replay()
        .canonical_commit_envelope(execution, feature_create.commit.commit_id)
        .unwrap()
        .expect("feature create envelope")
        .lineage_events()[0]
        .targets()
        .to_vec();
    assert_eq!(event_targets, vec![lineage_id]);

    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("sparse branch-local lineage recovers");
    let identity = recovered.branch_identity(&feature).unwrap();
    let (_, basis) = recovered.observe_branch(&identity).unwrap();
    assert_eq!(
        recovered
            .lineage_node_at_observation(execution, &basis.observation(), feature_entity)
            .unwrap()
            .unwrap()
            .node()
            .lineage_id(),
        lineage_id
    );
}
