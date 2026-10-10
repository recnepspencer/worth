use crate::facade::history::BranchId;
use crate::facade::merge::{
    DeletionExecutionClass, LoweredMergeBlockedReason, LoweredRecordDenialKind,
    MergeExecutionRequest, MergeIntent, MergeResolutionClass,
};
use crate::facade::transactions::RecordRef;
use crate::tests::support::{
    create_branch_from_main, create_entity, delete_entity, delete_entity_on_branch,
    persisted_runtime_with_test_schema, update_entity, update_entity_on_branch,
};

#[test]
fn lowered_plan_preserves_source_deleted_target_live_block_reason() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = persisted_runtime_with_test_schema();
    let entity = create_entity(&runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    delete_entity_on_branch(&runtime, entity, BranchId("feature".to_string()));

    assert_blocked_deletion_lowering(
        execution,
        &mut runtime,
        entity,
        LoweredMergeBlockedReason::SourceDeletedTargetLive,
        DeletionExecutionClass::SourceDeletedTargetLive,
        LoweredRecordDenialKind::BlockedSourceDeletedTargetLive,
    );
}

#[test]
fn lowered_plan_preserves_source_live_target_deleted_block_reason() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = persisted_runtime_with_test_schema();
    let entity = create_entity(&runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity_on_branch(&runtime, entity, "shared", BranchId("feature".to_string()));
    delete_entity(&runtime, entity);

    assert_blocked_deletion_lowering(
        execution,
        &mut runtime,
        entity,
        LoweredMergeBlockedReason::SourceLiveTargetDeleted,
        DeletionExecutionClass::SourceLiveTargetDeleted,
        LoweredRecordDenialKind::BlockedSourceLiveTargetDeleted,
    );
}

#[test]
fn lowered_plan_preserves_deleted_vs_modified_block_reason() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = persisted_runtime_with_test_schema();
    let entity = create_entity(&runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(&runtime, entity, "main-modified");
    delete_entity_on_branch(&runtime, entity, BranchId("feature".to_string()));

    assert_blocked_deletion_lowering(
        execution,
        &mut runtime,
        entity,
        LoweredMergeBlockedReason::DeletedVsModified,
        DeletionExecutionClass::DeletedVsModified,
        LoweredRecordDenialKind::BlockedDeletedVsModified,
    );
}

fn assert_blocked_deletion_lowering(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut crate::facade::runtime::RelationalRuntime,
    entity: crate::facade::identity::EntityId,
    blocked_reason: LoweredMergeBlockedReason,
    deletion_class: DeletionExecutionClass,
    denial_kind: LoweredRecordDenialKind,
) {
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

    let lowered = artifact
        .lowered_plan
        .records
        .iter()
        .find(|record| record.record == RecordRef::Entity(entity))
        .expect("lowered record");

    assert_eq!(lowered.blocked_reason, Some(blocked_reason));
    assert_eq!(
        lowered.resolution_class,
        MergeResolutionClass::Deletion(deletion_class)
    );
    assert_eq!(lowered.executable_class, None);

    let lowered_index = artifact
        .lowered_plan
        .records
        .iter()
        .position(|record| record.record == RecordRef::Entity(entity))
        .expect("lowered record index");

    assert_eq!(
        artifact.digest_basis.lowered_plan.blocked_reasons[lowered_index],
        Some(blocked_reason)
    );
    assert_eq!(
        artifact.digest_basis.lowered_plan.resolution_classes[lowered_index],
        MergeResolutionClass::Deletion(deletion_class)
    );
    assert_eq!(
        artifact.digest_basis.lowered_plan.executable_classes[lowered_index],
        None
    );
    assert_eq!(
        artifact.digest_basis.lowered_plan.denial_bundle_kinds[lowered_index],
        Some(denial_kind)
    );
}
