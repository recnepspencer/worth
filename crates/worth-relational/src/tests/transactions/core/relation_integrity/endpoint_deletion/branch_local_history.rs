use super::*;

#[test]
fn relation_integrity_endpoint_deletion_history_stays_branch_local_under_divergence() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, target, relation) =
        create_endpoint_deletion_relation_fixture(execution, &runtime, "live");

    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();

    let main_delete = delete_entity(execution, &runtime, source);
    let _feature_update = update_entity_on_branch(
        execution,
        &runtime,
        target,
        "feature-target",
        BranchId("feature".to_string()),
    );

    let main_digest = relation_aspect_history_digest_on_branch(
        execution,
        &runtime,
        &BranchId("main".to_string()),
        relation,
        None,
    );
    let feature_digest = relation_aspect_history_digest_on_branch(
        execution,
        &runtime,
        &BranchId("feature".to_string()),
        relation,
        None,
    );
    let main_head_version = runtime
        .history()
        .branch_head(&BranchId("main".to_string()))
        .unwrap()
        .version_id;
    let feature_head_version = runtime
        .history()
        .branch_head(&BranchId("feature".to_string()))
        .unwrap()
        .version_id;
    let main_inspection = runtime
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            &BranchId("main".to_string()),
            main_head_version,
            RecordRef::Relation(relation),
            crate::facade::inspection::HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");
    let feature_inspection = runtime
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            &BranchId("feature".to_string()),
            feature_head_version,
            RecordRef::Relation(relation),
            crate::facade::inspection::HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");

    assert_eq!(main_delete.version_id, main_head_version);
    assert_eq!(main_digest.entry_count, 2);
    assert!(feature_digest.entry_count < main_digest.entry_count);
    assert_eq!(
        main_inspection
            .aspect_history_observation
            .as_ref()
            .map(|observation| observation.query_result.trace.branch_id.clone()),
        Some(BranchId("main".to_string()))
    );
    assert_eq!(
        feature_inspection
            .aspect_history_observation
            .as_ref()
            .map(|observation| observation.query_result.trace.branch_id.clone()),
        Some(BranchId("feature".to_string()))
    );
    let main_read = runtime
        .read_truth()
        .read_snapshot(&main_delete.snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    assert_eq!(
        main_read.get_relation(relation).unwrap().lifecycle,
        RecordLifecycleState::RetainedDanglingForAudit
    );
}
