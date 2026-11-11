//! Shipping planner digest rows retain policy and visibility evidence shape.
use super::branch_history_publications::*;
use super::merge_history_planning::inspect_feature_reconciliation;
use worth_relational::facade::{history::BranchId, merge::MergePolicyOwnershipSurface};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};
#[test]
fn merge_planning_digest_basis_carries_visibility_evidence_rows() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let artifact = inspect_feature_reconciliation(execution, &runtime);

    assert_eq!(
        artifact.digest_basis.conflict.records.len(),
        artifact
            .digest_basis
            .conflict
            .source_visibility_evidence
            .len()
    );
    assert_eq!(
        artifact.digest_basis.conflict.records.len(),
        artifact
            .digest_basis
            .conflict
            .target_visibility_evidence
            .len()
    );
    assert_eq!(
        artifact.digest_basis.conflict.records.len(),
        artifact
            .digest_basis
            .conflict
            .base_visibility_evidence
            .len()
    );
}

#[test]
fn merge_planning_policy_surface_is_explicitly_runtime_owned_before_lowering() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    update_entity(execution, &runtime, entity, "same");
    update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "same",
        BranchId("feature".to_string()),
    );

    let artifact = inspect_feature_reconciliation(execution, &runtime);

    assert_eq!(
        artifact.policy_resolution.runtime_only_record_count,
        artifact.policy_resolution.resolved_record_count
    );
    assert_eq!(artifact.policy_resolution.custom_policy_record_count, 0);
}

#[test]
fn merge_planning_digest_basis_carries_policy_ownership_surface_rows() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let artifact = inspect_feature_reconciliation(execution, &runtime);

    assert_eq!(
        artifact.digest_basis.policy.records.len(),
        artifact.digest_basis.policy.proof_boundaries.len()
    );
    assert!(artifact
        .digest_basis
        .policy
        .proof_boundaries
        .iter()
        .all(|boundary| boundary.ownership_surface == MergePolicyOwnershipSurface::RuntimeOnly));
}
