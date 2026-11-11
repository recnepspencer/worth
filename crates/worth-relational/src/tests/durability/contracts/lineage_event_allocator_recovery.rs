use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
#[test]
fn branch_local_sparse_slot_create_publishes_and_recovers_exact_lineage() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "sparse-slot-shared-root");
    let feature = create_branch_from_main(&runtime, "sparse-slot-feature");
    create_entity_outcome(execution, &runtime, "sparse-slot-main-divergence");
    let feature_create = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "sparse-slot-feature-create",
        feature.clone(),
    );
    let feature_entity = changed_entities(&feature_create)[0];
    assert!(feature_entity.local_slot.0 > 1);
    let lineage_id = runtime
        .lineage
        .nodes_snapshot()
        .into_iter()
        .find(|node| node.entity_id() == feature_entity)
        .expect("sparse logical slot has lineage")
        .lineage_id();
    let event_targets = runtime
        .history()
        .demand_commit_envelope(execution, feature_create.commit.commit_id)
        .unwrap()
        .expect("feature create envelope")
        .lineage_events()[0]
        .targets()
        .to_vec();
    assert_eq!(event_targets, vec![lineage_id]);

    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("sparse branch-local lineage recovers");
    assert_eq!(
        recovered
            .lineage
            .nodes_snapshot()
            .into_iter()
            .find(|node| node.entity_id() == feature_entity)
            .expect("recovered sparse lineage")
            .lineage_id(),
        lineage_id
    );
}
