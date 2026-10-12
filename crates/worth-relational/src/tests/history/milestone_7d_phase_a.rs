use crate::facade::history::BranchId;
use crate::facade::merge::{
    MergeConflictClass, MergeExecutionRequest, MergeIntent, MergePolicyOwnershipSurface,
};
use crate::facade::transactions::RecordRef;
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch, delete_entity,
    delete_entity_on_branch, persisted_runtime_with_test_schema, update_entity,
    update_entity_on_branch,
};

#[test]
fn merge_planning_conflict_classification_carries_target_view_visibility_evidence_for_exact_shared_truth(
) {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(execution, &runtime, entity, "same");
    update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "same",
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

    let classification = artifact
        .conflict_classification
        .classifications
        .iter()
        .find(|classification| classification.record == RecordRef::Entity(entity))
        .expect("shared entity classification");

    assert_eq!(classification.class, MergeConflictClass::ExactSharedTruth);
    assert_eq!(
        classification.target_visibility_evidence.kind,
        crate::facade::merge::MergeVisibilityEvidenceKind::TargetCandidateViewLookup
    );
    assert_eq!(
        classification.target_visibility_evidence.state,
        crate::facade::merge::MergeVisibilityState::Visible
    );
    assert_eq!(
        classification
            .target_visibility_evidence
            .embedded_surface_state,
        Some(crate::facade::merge::MergeVisibilityState::Visible)
    );
    assert_eq!(
        classification.target_visibility_evidence.lifecycle,
        Some(crate::storage::data::RecordLifecycleState::Live)
    );
    assert!(classification
        .target_visibility_evidence
        .created_at_version
        .is_some());
}

#[test]
fn merge_planning_conflict_classification_carries_base_window_evidence_for_target_deleted_record() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "shared",
        BranchId("feature".to_string()),
    );
    delete_entity(execution, &runtime, entity);

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

    let classification = artifact
        .conflict_classification
        .classifications
        .iter()
        .find(|classification| classification.record == RecordRef::Entity(entity))
        .expect("deleted entity classification");

    assert!(matches!(
        classification.class,
        MergeConflictClass::Deletion(_)
    ));
    assert_eq!(
        classification.base_visibility_evidence.kind,
        crate::facade::merge::MergeVisibilityEvidenceKind::BaseResolvedViewLookup
    );
    assert_eq!(
        classification.base_visibility_evidence.state,
        crate::facade::merge::MergeVisibilityState::Visible
    );
    assert_eq!(
        classification.base_visibility_evidence.lifecycle,
        Some(crate::storage::data::RecordLifecycleState::Live)
    );
    assert_eq!(
        classification.target_visibility_evidence.kind,
        crate::facade::merge::MergeVisibilityEvidenceKind::TargetEmbeddedSurface
    );
    assert_eq!(
        classification.target_visibility_evidence.state,
        crate::facade::merge::MergeVisibilityState::NotVisible
    );
}

#[test]
fn merge_planning_conflict_classification_carries_target_lookup_evidence_for_source_deleted_record()
{
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

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

    let classification = artifact
        .conflict_classification
        .classifications
        .iter()
        .find(|classification| classification.record == RecordRef::Entity(entity))
        .expect("source-deleted entity classification");

    assert_eq!(
        classification.class,
        MergeConflictClass::Deletion(
            crate::facade::merge::DeletionMergeClass::SourceDeletedTargetLive
        )
    );
    assert_eq!(
        classification.source_visibility_evidence.kind,
        crate::facade::merge::MergeVisibilityEvidenceKind::SourceEmbeddedSurface
    );
    assert_eq!(
        classification.source_visibility_evidence.state,
        crate::facade::merge::MergeVisibilityState::NotVisible
    );
    assert_eq!(
        classification.target_visibility_evidence.kind,
        crate::facade::merge::MergeVisibilityEvidenceKind::TargetCandidateViewLookup
    );
    assert_eq!(
        classification.target_visibility_evidence.state,
        crate::facade::merge::MergeVisibilityState::Visible
    );
    assert_eq!(
        classification
            .target_visibility_evidence
            .embedded_surface_state,
        Some(crate::facade::merge::MergeVisibilityState::Visible)
    );
    assert_eq!(
        classification.base_visibility_evidence.kind,
        crate::facade::merge::MergeVisibilityEvidenceKind::BaseResolvedViewLookup
    );
    assert_eq!(
        classification.base_visibility_evidence.state,
        crate::facade::merge::MergeVisibilityState::Visible
    );
    assert_eq!(
        classification.base_visibility_evidence.lifecycle,
        Some(crate::storage::data::RecordLifecycleState::Live)
    );

    let main_head_version = runtime
        .history()
        .branch_head(&BranchId("main".to_string()))
        .expect("main branch head")
        .version_id;
    let main_view = runtime
        .read_truth()
        .read_version(execution, main_head_version);
    let target_record = main_view
        .get_entity(entity)
        .expect("main branch should still see entity at its head");
    assert_eq!(
        target_record.lifecycle,
        crate::storage::data::RecordLifecycleState::Live
    );
}

#[test]
fn merge_planning_digest_basis_carries_visibility_evidence_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(execution, &runtime, entity, "same");
    update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "same",
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

    assert_eq!(
        artifact.policy_resolution.runtime_only_record_count,
        artifact.policy_resolution.resolved_record_count
    );
    assert_eq!(artifact.policy_resolution.custom_policy_record_count, 0);
}

#[test]
fn merge_planning_digest_basis_carries_policy_ownership_surface_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
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
