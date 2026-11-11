//! Exact published node associations and canonical Create events survive source recovery.
use super::checkpoint_source_capture::capture_durably;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::RecoveryVerificationMode,
    identity::{EntityId, LineageId},
    lineage::LineageEventKind,
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime},
};
fn lineage(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
) -> LineageId {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .lineage_node_at_observation(execution, &basis.observation(), entity)
        .unwrap()
        .unwrap()
        .node()
        .lineage_id()
}
#[test]
fn checkpoint_source_recovers_each_original_lineage_node_and_create_event() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let first = create_entity_outcome(execution, &runtime, "first");
    let second = create_entity_outcome(execution, &runtime, "second");
    let first_entity = first.semantic_result().created_entities()[0].assigned;
    let second_entity = second.semantic_result().created_entities()[0].assigned;
    let first_lineage = lineage(execution, &runtime, first_entity);
    let second_lineage = lineage(execution, &runtime, second_entity);
    assert_ne!(first_lineage, second_lineage);
    capture_durably(execution, &runtime, &medium).unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(outcome.recovered_commits, 2);
    let mut event_count = 0;
    for receipt in [&first.commit, &second.commit] {
        let source = runtime
            .replay()
            .canonical_commit_envelope(execution, receipt.commit_id)
            .unwrap()
            .unwrap();
        let restored = recovered
            .replay()
            .canonical_commit_envelope(execution, receipt.commit_id)
            .unwrap()
            .unwrap();
        assert_eq!(restored.lineage_events(), source.lineage_events());
        assert!(restored
            .lineage_events()
            .iter()
            .all(|event| event.kind() == LineageEventKind::Create));
        event_count += restored.lineage_events().len();
    }
    assert_eq!(event_count, 2);
    assert_eq!(lineage(execution, &recovered, first_entity), first_lineage);
    assert_eq!(
        lineage(execution, &recovered, second_entity),
        second_lineage
    );
}
