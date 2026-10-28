use super::*;

#[test]
fn deleted_on_both_sides_prepared_merge_rejects_target_head_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    delete_entity(execution, &runtime, entity);
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
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
fn deleted_on_both_sides_prepared_merge_rejects_schema_semantic_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = durable_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    delete_entity(execution, &runtime, entity);
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge");
    runtime.set_schema_registry_for_test(drifted_schema_registry());

    match runtime.execute_prepared_merge(execution, prepared) {
        Err(MergeExecutionError::SchemaSemanticDrift { .. }) => {}
        other => panic!("expected schema semantic drift rejection, got {other:?}"),
    }
}

#[test]
fn non_executable_deletion_denial_is_stable_across_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(execution, &runtime, entity, "main-modified");
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let live_artifact = runtime
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
        checkpoint_and_recover_with(execution, &runtime, durable_runtime_with_test_schema);
    let recovered_artifact = recovered
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
