//! Shipping planner visibility evidence over actual durable branch publications.
use super::branch_history_publications::*;
use super::branch_history_recovery::current_head;
use super::merge_history_planning::inspect_feature_reconciliation;
use worth_relational::facade::{
    history::BranchId, merge::MergeConflictClass, transactions::RecordRef,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};
#[test]
fn merge_planning_conflict_classification_carries_target_view_visibility_evidence_for_exact_shared_truth(
) {
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

    let classification = artifact
        .conflict_classification
        .classifications
        .iter()
        .find(|classification| classification.record == RecordRef::Entity(entity))
        .expect("shared entity classification");

    assert_eq!(classification.class, MergeConflictClass::ExactSharedTruth);
    assert_eq!(
        classification.target_visibility_evidence.kind,
        worth_relational::facade::merge::MergeVisibilityEvidenceKind::TargetCandidateViewLookup
    );
    assert_eq!(
        classification.target_visibility_evidence.state,
        worth_relational::facade::merge::MergeVisibilityState::Visible
    );
    assert_eq!(
        classification
            .target_visibility_evidence
            .embedded_surface_state,
        Some(worth_relational::facade::merge::MergeVisibilityState::Visible)
    );
    assert_eq!(
        classification.target_visibility_evidence.lifecycle,
        Some(worth_relational::facade::storage::RecordLifecycleState::Live)
    );
    assert!(classification
        .target_visibility_evidence
        .created_at_version
        .is_some());
}

#[test]
fn merge_planning_conflict_classification_carries_base_window_evidence_for_target_deleted_record() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "shared",
        BranchId("feature".to_string()),
    );
    delete_entity(execution, &runtime, entity);

    let artifact = inspect_feature_reconciliation(execution, &runtime);

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
        worth_relational::facade::merge::MergeVisibilityEvidenceKind::BaseResolvedViewLookup
    );
    assert_eq!(
        classification.base_visibility_evidence.state,
        worth_relational::facade::merge::MergeVisibilityState::Visible
    );
    assert_eq!(
        classification.base_visibility_evidence.lifecycle,
        Some(worth_relational::facade::storage::RecordLifecycleState::Live)
    );
    assert_eq!(
        classification.target_visibility_evidence.kind,
        worth_relational::facade::merge::MergeVisibilityEvidenceKind::TargetEmbeddedSurface
    );
    assert_eq!(
        classification.target_visibility_evidence.state,
        worth_relational::facade::merge::MergeVisibilityState::NotVisible
    );
}

#[test]
fn merge_planning_conflict_classification_carries_target_lookup_evidence_for_source_deleted_record()
{
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let artifact = inspect_feature_reconciliation(execution, &runtime);

    let classification = artifact
        .conflict_classification
        .classifications
        .iter()
        .find(|classification| classification.record == RecordRef::Entity(entity))
        .expect("source-deleted entity classification");

    assert_eq!(
        classification.class,
        MergeConflictClass::Deletion(
            worth_relational::facade::merge::DeletionMergeClass::SourceDeletedTargetLive
        )
    );
    assert_eq!(
        classification.source_visibility_evidence.kind,
        worth_relational::facade::merge::MergeVisibilityEvidenceKind::SourceEmbeddedSurface
    );
    assert_eq!(
        classification.source_visibility_evidence.state,
        worth_relational::facade::merge::MergeVisibilityState::NotVisible
    );
    assert_eq!(
        classification.target_visibility_evidence.kind,
        worth_relational::facade::merge::MergeVisibilityEvidenceKind::TargetCandidateViewLookup
    );
    assert_eq!(
        classification.target_visibility_evidence.state,
        worth_relational::facade::merge::MergeVisibilityState::Visible
    );
    assert_eq!(
        classification
            .target_visibility_evidence
            .embedded_surface_state,
        Some(worth_relational::facade::merge::MergeVisibilityState::Visible)
    );
    assert_eq!(
        classification.base_visibility_evidence.kind,
        worth_relational::facade::merge::MergeVisibilityEvidenceKind::BaseResolvedViewLookup
    );
    assert_eq!(
        classification.base_visibility_evidence.state,
        worth_relational::facade::merge::MergeVisibilityState::Visible
    );
    assert_eq!(
        classification.base_visibility_evidence.lifecycle,
        Some(worth_relational::facade::storage::RecordLifecycleState::Live)
    );

    let main_head_version = current_head(&runtime, &BranchId("main".into()))
        .expect("main branch head")
        .version_id;
    let main_view = runtime
        .read_truth()
        .try_read_version(execution, main_head_version)
        .expect("read request remains live")
        .expect("the actual target head remains retained");
    let target_record = main_view
        .get_entity(entity)
        .expect("main branch should still see entity at its head");
    assert_eq!(
        target_record.lifecycle,
        worth_relational::facade::storage::RecordLifecycleState::Live
    );
}
