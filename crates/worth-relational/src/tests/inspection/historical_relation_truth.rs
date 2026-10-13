use super::*;

#[test]
fn historical_relation_inspection_keeps_direct_commit_history_when_retained_only_blocks_record_truth(
) {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let relation_outcome =
        create_relation_outcome(execution, &runtime, source, target, "historical-rel");
    let relation = crate::tests::support::changed_relations(&relation_outcome)[0];
    let _later = create_entity_outcome(execution, &runtime, "later");
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&relation_outcome.snapshot)
        .is_ok());

    let inspection = runtime
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            &BranchId("main".to_string()),
            relation_outcome.version_id,
            crate::facade::transactions::RecordRef::Relation(relation),
            HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");

    assert_eq!(
        inspection.record_observation.availability,
        InspectionAvailability::UnavailableByRetention
    );
    assert!(inspection.record_observation.value.is_none());
    assert!(inspection.lineage_resolution_context.is_none());
    assert!(inspection.structural_identity_evidence.is_none());
    assert_eq!(
        inspection
            .aspect_history_observation
            .as_ref()
            .map(|observation| observation.availability),
        Some(InspectionAvailability::Direct)
    );
    assert_eq!(
        inspection
            .aspect_history_observation
            .as_ref()
            .map(|observation| observation.query_result.trace.branch_id.clone()),
        Some(BranchId("main".to_string()))
    );
}

#[test]
fn historical_relation_inspection_reconstructs_record_truth_without_inventing_lineage() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .visibility_cache_policy(VisibilityCachePolicy {
            enabled: true,
            protect_branch_heads: false,
            protect_replay_retained: false,
            protect_active_snapshots: false,
            recent_version_window: 0,
        })
        .build();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let relation_outcome =
        create_relation_outcome(execution, &runtime, source, target, "reconstructed-rel");
    let relation = crate::tests::support::changed_relations(&relation_outcome)[0];
    let _later = create_entity_outcome(execution, &runtime, "later");
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&relation_outcome.snapshot)
        .is_ok());

    let inspection = runtime
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            &BranchId("main".to_string()),
            relation_outcome.version_id,
            crate::facade::transactions::RecordRef::Relation(relation),
            HistoricalInspectionMode::AllowCanonicalReconstruction,
        )
        .expect("read request remains live");

    assert_eq!(
        inspection.record_observation.availability,
        InspectionAvailability::Reconstructed
    );
    match inspection.record_observation.value {
        Some(crate::facade::inspection::HistoricalRecordValue::Relation(ref record)) => {
            assert_eq!(record.relation_id, relation);
            assert_eq!(record.source, source);
            assert_eq!(record.target, target);
        }
        _ => panic!("expected reconstructed relation record"),
    }
    assert!(inspection.lineage_resolution_context.is_none());
    let structural = inspection
        .structural_identity_evidence
        .as_ref()
        .expect("relation structural evidence");
    assert_eq!(
        structural.availability,
        InspectionAvailability::Reconstructed
    );
    assert!(structural.structural_fingerprint.is_none());
    assert!(structural.lineage_id.is_none());
    assert!(structural
        .degradations
        .contains(&crate::facade::inspection::InspectionDegradation::MissingStructuralFingerprint));
    assert!(structural
        .degradations
        .contains(&crate::facade::inspection::InspectionDegradation::MissingLineageIdentity));
}
