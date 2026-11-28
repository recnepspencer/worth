use super::*;

#[test]
fn commit_inspection_is_canonical_and_not_story_shaped() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "commit-inspection");
    let commit_id = runtime
        .history()
        .latest_commit()
        .map(|commit| commit.commit_id)
        .expect("latest commit");

    let inspection = runtime
        .inspect_what_happened()
        .inspect_commit(execution, commit_id)
        .unwrap()
        .expect("commit inspection");
    let history = runtime.history();
    let envelope = history
        .demand_commit_envelope(execution, commit_id)
        .unwrap()
        .expect("commit envelope");

    assert_eq!(inspection.origin, InspectionOrigin::CanonicalCommitStorage);
    assert_eq!(
        inspection.access_path,
        InspectionAccessPath::CommitIndexRead
    );
    assert_eq!(inspection.commit.commit_id, commit_id);
    assert_eq!(
        inspection.changed_records,
        vec![crate::facade::transactions::RecordRef::Entity(entity)]
    );
    assert_eq!(inspection.lineage_event_ids, envelope.lineage_event_ids());
    assert_eq!(inspection.lineage_events, envelope.lineage_events());
    assert_eq!(
        inspection.lineage_digest_basis,
        *envelope.lineage_digest_basis()
    );
    assert_eq!(
        inspection.lineage_artifact_counters,
        envelope.lineage_artifact_counters()
    );
    assert_eq!(
        inspection.derived_index_artifacts,
        *envelope.derived_index_artifacts()
    );
    assert_eq!(
        inspection.changed_aspects,
        crate::publication::patch::data::ordered_aspect_keys(
            envelope
                .patch
                .authoritative_record_patches
                .iter()
                .flat_map(|record| record.authoritative_changed_aspect_keys().cloned())
        )
    );
}
