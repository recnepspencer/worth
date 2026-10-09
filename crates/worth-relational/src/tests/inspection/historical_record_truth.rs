use super::*;

#[test]
fn historical_record_inspection_and_transaction_staging_are_read_only() {
    let runtime = runtime_with_test_schema();
    let created = create_entity(&runtime, "staged");
    let commit_id = runtime
        .history()
        .latest_commit()
        .map(|commit| commit.commit_id)
        .unwrap_or(CommitId(0));

    let historical = retained_record_inspection(
        &runtime,
        &crate::facade::history::BranchId("main".to_string()),
        runtime.current_version_id(),
        crate::facade::transactions::RecordRef::Entity(created),
    );
    assert_eq!(
        historical.record_observation.availability,
        InspectionAvailability::Direct
    );

    let commit = runtime
        .inspect_what_happened()
        .inspect_commit(commit_id)
        .expect("commit inspection");
    assert!(commit
        .changed_records
        .contains(&crate::facade::transactions::RecordRef::Entity(created)));

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        batch_create("pending"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let staging = txn.inspect_staging();
    assert_eq!(staging.batch_count, 1);
    assert!(staging.touched_records.is_empty());

    let neighbors: NeighborInspectionResult = runtime
        .inspect_what_happened()
        .neighbors(InspectionScope::Current, created);
    assert!(neighbors.outgoing_relation_ids.is_empty());
    assert!(neighbors.incoming_relation_ids.is_empty());
}

#[test]
fn historical_record_inspection_preserves_requested_branch_context() {
    let runtime = runtime_with_test_schema();
    let created = create_entity(&runtime, "branch-base");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .expect("feature branch");

    let inspection = runtime.inspect_what_happened().inspect_historical_record(
        &BranchId("feature".to_string()),
        runtime.current_version_id(),
        crate::facade::transactions::RecordRef::Entity(created),
        HistoricalInspectionMode::RetainedOnly,
    );

    assert_eq!(inspection.branch_id, BranchId("feature".to_string()));
    assert_eq!(
        inspection
            .lineage_resolution_context
            .as_ref()
            .map(|resolution| resolution.branch_id.clone()),
        Some(BranchId("feature".to_string()))
    );
    assert_eq!(
        inspection
            .aspect_history_observation
            .as_ref()
            .map(|observation| observation.query_result.trace.branch_id.clone()),
        Some(BranchId("feature".to_string()))
    );
}

#[test]
fn adjacency_truth_uses_the_explicit_version_instead_of_current_head() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let first_target = create_entity(&runtime, "first-target");
    let first = create_relation_outcome(&runtime, source, first_target, "first");
    let retained_version = first.version_id;
    let second_target = create_entity(&runtime, "second-target");
    let second = create_relation_outcome(&runtime, source, second_target, "second");
    let relation_kind = crate::facade::identity::KindId(2);

    let historical = runtime.read_truth().outgoing_relations_of_kind_at_version(
        source,
        relation_kind,
        retained_version,
    );
    let current = runtime.read_truth().outgoing_relations_of_kind_at_version(
        source,
        relation_kind,
        runtime.current_version_id(),
    );
    let historical_incoming = runtime.read_truth().incoming_relations_of_kind_at_version(
        first_target,
        relation_kind,
        retained_version,
    );

    assert_eq!(historical.len(), 1);
    assert_eq!(historical[0].target, first_target);
    assert_eq!(current.len(), 2);
    assert_eq!(historical_incoming.len(), 1);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&first.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&second.snapshot)
        .is_ok());
}

#[test]
fn historical_open_fails_closed_without_retained_state_and_reconstructs_canonically_when_allowed() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "historical");
    let later = create_entity_outcome(&runtime, "later");
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&later.snapshot)
        .is_ok());

    let retained_only = runtime
        .inspect_what_happened()
        .open_historical_view(created.version_id, HistoricalInspectionMode::RetainedOnly);
    assert!(retained_only.view.is_none());
    assert_eq!(
        retained_only.availability,
        InspectionAvailability::UnavailableByRetention
    );

    let reconstructed = runtime.inspect_what_happened().inspect_historical_record(
        &BranchId("main".to_string()),
        created.version_id,
        crate::facade::transactions::RecordRef::Entity(changed_entities(&created)[0]),
        HistoricalInspectionMode::AllowCanonicalReconstruction,
    );
    assert_eq!(
        reconstructed.record_observation.availability,
        InspectionAvailability::Reconstructed
    );
    assert!(reconstructed.record_observation.value.is_some());
    assert_eq!(
        reconstructed
            .retention_availability_observation
            .as_ref()
            .map(|observation| observation.retained_directly),
        Some(false)
    );
}

#[test]
fn historical_record_inspection_keeps_subresults_separate_when_retained_only_blocks_record_truth() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "historical");
    let entity = changed_entities(&created)[0];
    let _updated = create_entity_outcome(&runtime, "later");
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());

    let inspection = runtime.inspect_what_happened().inspect_historical_record(
        &BranchId("main".to_string()),
        created.version_id,
        crate::facade::transactions::RecordRef::Entity(entity),
        HistoricalInspectionMode::RetainedOnly,
    );

    assert_eq!(
        inspection.record_observation.availability,
        InspectionAvailability::UnavailableByRetention
    );
    assert!(inspection.record_observation.value.is_none());
    assert!(inspection.structural_identity_evidence.is_none());
    assert_eq!(
        inspection
            .retention_availability_observation
            .as_ref()
            .map(|observation| observation.availability),
        Some(InspectionAvailability::UnavailableByRetention)
    );
    assert_eq!(
        inspection
            .lineage_resolution_context
            .as_ref()
            .map(|resolution| resolution.branch_id.clone()),
        Some(BranchId("main".to_string()))
    );
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
            .map(|observation| observation.access_path),
        Some(InspectionAccessPath::CommitIndexRead)
    );
}

#[test]
fn historical_inspection_matrix_keeps_entity_and_relation_subresults_honest_across_modes() {
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
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let relation_outcome = create_relation_outcome(&runtime, source, target, "matrix-rel");
    let relation = crate::tests::support::changed_relations(&relation_outcome)[0];
    let entity_version = runtime.current_version_id();
    let _later = create_entity_outcome(&runtime, "later");
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&relation_outcome.snapshot)
        .is_ok());

    let entity_retained = runtime.inspect_what_happened().inspect_historical_record(
        &BranchId("main".to_string()),
        entity_version,
        crate::facade::transactions::RecordRef::Entity(source),
        HistoricalInspectionMode::RetainedOnly,
    );
    let relation_retained = runtime.inspect_what_happened().inspect_historical_record(
        &BranchId("main".to_string()),
        relation_outcome.version_id,
        crate::facade::transactions::RecordRef::Relation(relation),
        HistoricalInspectionMode::RetainedOnly,
    );
    let entity_reconstructed = runtime.inspect_what_happened().inspect_historical_record(
        &BranchId("main".to_string()),
        entity_version,
        crate::facade::transactions::RecordRef::Entity(source),
        HistoricalInspectionMode::AllowCanonicalReconstruction,
    );
    let relation_reconstructed = runtime.inspect_what_happened().inspect_historical_record(
        &BranchId("main".to_string()),
        relation_outcome.version_id,
        crate::facade::transactions::RecordRef::Relation(relation),
        HistoricalInspectionMode::AllowCanonicalReconstruction,
    );

    assert_eq!(
        entity_retained.record_observation.availability,
        InspectionAvailability::UnavailableByRetention
    );
    assert!(entity_retained.lineage_resolution_context.is_some());
    assert_eq!(
        entity_retained
            .aspect_history_observation
            .as_ref()
            .map(|observation| observation.availability),
        Some(InspectionAvailability::Direct)
    );
    assert!(entity_retained.structural_identity_evidence.is_none());

    assert_eq!(
        relation_retained.record_observation.availability,
        InspectionAvailability::UnavailableByRetention
    );
    assert!(relation_retained.lineage_resolution_context.is_none());
    assert_eq!(
        relation_retained
            .aspect_history_observation
            .as_ref()
            .map(|observation| observation.availability),
        Some(InspectionAvailability::Direct)
    );
    assert!(relation_retained.structural_identity_evidence.is_none());

    assert_eq!(
        entity_reconstructed.record_observation.availability,
        InspectionAvailability::Reconstructed
    );
    assert!(entity_reconstructed.lineage_resolution_context.is_some());
    assert!(entity_reconstructed.structural_identity_evidence.is_some());

    assert_eq!(
        relation_reconstructed.record_observation.availability,
        InspectionAvailability::Reconstructed
    );
    assert!(relation_reconstructed.lineage_resolution_context.is_none());
    assert!(relation_reconstructed
        .structural_identity_evidence
        .is_some());
}
