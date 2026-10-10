use crate::facade::history::BranchId;
use crate::facade::merge::{
    AspectMergePolicyKind, MergeExecutionRequest, MergeIntent, MergeManualResolutionClass,
    MergePolicyDecisionBoundary, MergePolicyOwnershipSurface, MergePolicyRejectClass,
};
use crate::facade::transactions::RecordRef;
use crate::tests::support::{
    create_branch_from_main, create_entity, delete_entity_on_branch,
    persisted_runtime_with_test_schema, update_entity, update_entity_on_branch,
};

use super::fixtures::runtime_with_name_merge_policy;

#[test]
fn lowered_plan_carries_explicit_manual_resolution_policy_boundary_for_generic_denial() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let entity = create_entity(&runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    delete_entity_on_branch(&runtime, entity, BranchId("feature".to_string()));

    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            }
            .into(),
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_name_merge_policy(AspectMergePolicyKind::FailOnConflict);
    let entity = create_entity(&runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(&runtime, entity, "main-name");
    update_entity_on_branch(
        &runtime,
        entity,
        "feature-name",
        BranchId("feature".to_string()),
    );

    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            }
            .into(),
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
        crate::facade::merge::LoweredRecordDecision::Reject(_)
    ));
}
