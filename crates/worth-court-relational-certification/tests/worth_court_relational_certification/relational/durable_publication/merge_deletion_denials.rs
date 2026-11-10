//! Deletion merges: stale mutual-deletion execution and recovered non-executable denial.
use super::branch_history_publications::{
    create_entity, delete_entity, delete_entity_on_branch, fork, update_entity,
};
use super::merge_execution_observation::diagnostic_field;
use super::merge_execution_recovery::checkpoint_and_recover_durable;
use super::merge_proof_journey::merge_request;
use worth_relational::facade::diagnostics::{DiagnosticCode, RelationalDiagnosticValue};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::merge::{
    DeletionExecutionClass, LoweredMergeBlockedReason, LoweredRecordDenialKind,
    MergeExecutionError, MergeResolutionClass,
};
use worth_relational::facade::transactions::RecordRef;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::durable_runtime;

#[test]
fn deleted_on_both_sides_prepared_merge_rejects_target_head_drift() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    delete_entity(execution, &runtime, entity);
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .expect("prepared merge");
    create_entity(execution, &runtime, "main-advance");

    match runtime.execute_prepared_merge(execution, prepared) {
        Err(MergeExecutionError::StaleBranchHead { branch, .. }) => {
            assert_eq!(branch, BranchId("main".to_string()));
        }
        other => panic!("expected target stale-head rejection, got {other:?}"),
    }

    let diagnostics = runtime.publication().diagnostics();
    let failure_artifact = diagnostics
        .artifacts()
        .iter()
        .rev()
        .find(|artifact| {
            artifact.entries.iter().any(|entry| {
                entry.code == DiagnosticCode::DeterministicMergeViolation
                    || entry.code == DiagnosticCode::MissingMergeBase
            })
        })
        .expect("failure artifact");
    assert!(failure_artifact.entries.iter().any(|entry| {
        entry.code == DiagnosticCode::DeterministicMergeViolation
            && diagnostic_field(entry, "target_branch")
                == &RelationalDiagnosticValue::BranchId(BranchId("main".to_string()))
            && diagnostic_field(entry, "source_branch")
                == &RelationalDiagnosticValue::BranchId(BranchId("feature".to_string()))
    }));
}

#[test]
fn non_executable_deletion_denial_is_stable_across_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    update_entity(execution, &runtime, entity, "main-modified");
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let live_artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
        )
        .expect("live planning artifact");
    let live_record = live_artifact
        .lowered_plan
        .records
        .iter()
        .find(|record| record.record == RecordRef::Entity(entity))
        .expect("live lowered record");
    let live_index = live_artifact
        .lowered_plan
        .records
        .iter()
        .position(|record| record.record == RecordRef::Entity(entity))
        .expect("live lowered index");

    let (_recovery, recovered) =
        checkpoint_and_recover_durable(execution, &runtime, &medium, durable_runtime(&medium));
    let recovered_artifact = recovered
        .merge()
        .inspect_planning_scope(
            execution,
            recovered
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
        )
        .expect("recovered planning artifact");
    let recovered_record = recovered_artifact
        .lowered_plan
        .records
        .iter()
        .find(|record| record.record == RecordRef::Entity(entity))
        .expect("recovered lowered record");
    let recovered_index = recovered_artifact
        .lowered_plan
        .records
        .iter()
        .position(|record| record.record == RecordRef::Entity(entity))
        .expect("recovered lowered index");

    assert_eq!(
        live_record.resolution_class,
        MergeResolutionClass::Deletion(DeletionExecutionClass::DeletedVsModified)
    );
    assert_eq!(
        live_record.blocked_reason,
        Some(LoweredMergeBlockedReason::DeletedVsModified)
    );
    assert_eq!(live_record.executable_class, None);
    assert_eq!(live_record, recovered_record);
    assert_eq!(
        live_artifact.digest_basis.lowered_plan.denial_bundle_kinds[live_index],
        Some(LoweredRecordDenialKind::BlockedDeletedVsModified)
    );
    assert_eq!(
        recovered_artifact
            .digest_basis
            .lowered_plan
            .denial_bundle_kinds[recovered_index],
        Some(LoweredRecordDenialKind::BlockedDeletedVsModified)
    );
    assert_eq!(
        live_artifact.digest_basis.lowered_plan,
        recovered_artifact.digest_basis.lowered_plan
    );
}
