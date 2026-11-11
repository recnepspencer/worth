//! Merge planning artifact equality across durable checkpoint and recovery over one explicit medium.
use super::branch_history_publications::{
    create_entity, fork, update_entity, update_entity_on_branch,
};
use super::merge_execution_recovery::checkpoint_and_recover_durable;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::merge::{
    MergeIntent, MergePlanningArtifactCore, MergePlanningError, MergePlanningRequest,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::durable_runtime;

/// Plan main <- feature through the owner binding of the runtime that plans it.
fn inspect_planning_scope(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Result<MergePlanningArtifactCore, MergePlanningError> {
    let request = MergePlanningRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    );
    runtime.merge().inspect_planning_scope(
        execution,
        runtime.bind_merge_planning_request(request).unwrap(),
    )
}

/// The live artifact, after the receiver recovered from the caller's medium
/// reproduced it exactly.
fn inspect_recovered_merge_planning_artifact(
    execution: &ExecutionRequest<'_, '_>,
    medium: &Arc<TestDurableMedium>,
) -> MergePlanningArtifactCore {
    let runtime = durable_runtime(medium);
    let shared = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    update_entity(execution, &runtime, shared, "shared-main");
    update_entity_on_branch(
        execution,
        &runtime,
        shared,
        "shared-feature",
        BranchId("feature".to_string()),
    );

    let artifact =
        inspect_planning_scope(execution, &runtime).expect("live merge planning artifact");

    let (_recovery, recovered) =
        checkpoint_and_recover_durable(execution, &runtime, medium, durable_runtime(medium));
    let recovered_artifact =
        inspect_planning_scope(execution, &recovered).expect("recovered merge planning artifact");

    assert_eq!(recovered_artifact, artifact);
    artifact
}

#[test]
fn merge_planning_artifact_certification_is_stable_across_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let artifact = inspect_recovered_merge_planning_artifact(execution, &medium);

    assert_eq!(artifact.digest_basis.schema, artifact.schema_snapshot);
    assert_eq!(
        artifact.digest_basis.decision_log,
        artifact.decision_log_digest_basis
    );
    assert_eq!(
        artifact.digest_basis.lowered_plan.records.len(),
        artifact.lowered_plan.records.len()
    );
    assert_eq!(
        artifact.decision_log_digest_basis.canonical_records.len(),
        artifact.decision_log.decisions.len()
    );
    assert!(!artifact.schema_snapshot.touched_kinds.is_empty());
    assert!(artifact.lowered_plan.record_count > 0);
    assert!(artifact.summary.request_summary.contains("main"));
}
