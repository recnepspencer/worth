//! Lowered merge plans keep the typed block reason of each one-sided deletion.
use super::branch_history_publications::{
    create_entity, delete_entity, delete_entity_on_branch, fork, update_entity,
    update_entity_on_branch,
};
use super::merge_proof_journey::merge_request;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::merge::{
    DeletionExecutionClass, LoweredMergeBlockedReason, LoweredRecordDenialKind,
    MergeResolutionClass,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::RecordRef;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::durable_runtime;

#[test]
fn lowered_plan_preserves_source_deleted_target_live_block_reason() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    assert_blocked_deletion_lowering(
        execution,
        &runtime,
        entity,
        LoweredMergeBlockedReason::SourceDeletedTargetLive,
        DeletionExecutionClass::SourceDeletedTargetLive,
        LoweredRecordDenialKind::BlockedSourceDeletedTargetLive,
    );
}

#[test]
fn lowered_plan_preserves_source_live_target_deleted_block_reason() {
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

    assert_blocked_deletion_lowering(
        execution,
        &runtime,
        entity,
        LoweredMergeBlockedReason::SourceLiveTargetDeleted,
        DeletionExecutionClass::SourceLiveTargetDeleted,
        LoweredRecordDenialKind::BlockedSourceLiveTargetDeleted,
    );
}

#[test]
fn lowered_plan_preserves_deleted_vs_modified_block_reason() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    update_entity(execution, &runtime, entity, "main-modified");
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    assert_blocked_deletion_lowering(
        execution,
        &runtime,
        entity,
        LoweredMergeBlockedReason::DeletedVsModified,
        DeletionExecutionClass::DeletedVsModified,
        LoweredRecordDenialKind::BlockedDeletedVsModified,
    );
}

fn assert_blocked_deletion_lowering(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
    blocked_reason: LoweredMergeBlockedReason,
    deletion_class: DeletionExecutionClass,
    denial_kind: LoweredRecordDenialKind,
) {
    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
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
