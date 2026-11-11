//! Lowered merge plans carry the explicit runtime-owned policy proof boundary of each denial.
use super::branch_history_publications::{
    create_entity, delete_entity_on_branch, fork, update_entity, update_entity_on_branch,
};
use super::merge_execution_schema::runtime_with_name_merge_policy;
use super::merge_proof_journey::merge_request;
use worth_relational::facade::{
    history::BranchId,
    merge::{
        AspectMergePolicyKind, LoweredRecordDecision, MergeManualResolutionClass,
        MergePolicyDecisionBoundary, MergePolicyOwnershipSurface, MergePolicyRejectClass,
    },
    transactions::RecordRef,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};

#[test]
fn lowered_plan_carries_explicit_manual_resolution_policy_boundary_for_generic_denial() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
        )
        .expect("planning artifact");

    let lowered_index = artifact
        .lowered_plan
        .records
        .iter()
        .position(|record| record.record == RecordRef::Entity(entity))
        .expect("lowered record index");
    let lowered = &artifact.lowered_plan.records[lowered_index];

    assert_eq!(
        lowered.policy_proof_boundary.ownership_surface,
        MergePolicyOwnershipSurface::RuntimeOnly
    );
    assert_eq!(
        lowered.policy_proof_boundary.decision_boundary,
        MergePolicyDecisionBoundary::RequiresManualResolution {
            class: MergeManualResolutionClass::GenericRuntimeConflict,
        }
    );
    assert_eq!(
        artifact.digest_basis.policy.proof_boundaries[lowered_index],
        lowered.policy_proof_boundary
    );
}

#[test]
fn lowered_plan_carries_explicit_hard_reject_policy_boundary_for_fail_on_conflict() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_name_merge_policy(AspectMergePolicyKind::FailOnConflict);
    let entity = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    update_entity(execution, &runtime, entity, "main-name");
    update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "feature-name",
        BranchId("feature".to_string()),
    );

    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
        )
        .expect("planning artifact");

    let lowered_index = artifact
        .lowered_plan
        .records
        .iter()
        .position(|record| record.record == RecordRef::Entity(entity))
        .expect("lowered record index");
    let lowered = &artifact.lowered_plan.records[lowered_index];

    assert_eq!(
        lowered.policy_proof_boundary.ownership_surface,
        MergePolicyOwnershipSurface::RuntimeOnly
    );
    assert_eq!(
        lowered.policy_proof_boundary.decision_boundary,
        MergePolicyDecisionBoundary::Reject {
            class: MergePolicyRejectClass::BuiltInFailOnConflict,
        }
    );
    assert_eq!(
        artifact.digest_basis.policy.proof_boundaries[lowered_index],
        lowered.policy_proof_boundary
    );
    assert!(matches!(
        lowered.record_decision,
        LoweredRecordDecision::Reject(_)
    ));
}
