use super::*;

#[test]
fn commit_inspection_is_canonical_and_not_story_shaped() {
    let runtime = runtime_with_test_schema();
    let entity = create_entity(&runtime, "commit-inspection");
    let commit_id = runtime
        .history()
        .latest_commit()
        .map(|commit| commit.commit_id)
        .expect("latest commit");

    let inspection = runtime
        .inspect_what_happened()
        .inspect_commit(commit_id)
        .expect("commit inspection");
    let history = runtime.history();
    let envelope = history.commit_envelope(commit_id).expect("commit envelope");

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

#[test]
fn merge_commit_inspection_stays_envelope_projected() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "merge-base");
    let entity = changed_entities(&created)[0];
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .expect("feature branch");

    let mut feature_txn = crate::tests::support::test_owner_begin_transaction_for_branch(
        &runtime,
        BranchId("feature".to_string()),
    );
    feature_txn
        .push_batch(
            WorkerIntentBatch::new("feature-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "feature",
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    feature_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("feature update");

    let merge = merge_commit_from_branches(
        &runtime,
        BranchId("main".to_string()),
        vec![BranchId("feature".to_string())],
    );
    let merge_commit_id = merge.commit.commit_id;
    let history = runtime.history();
    let envelope = history
        .commit_envelope(merge_commit_id)
        .expect("merge commit envelope");
    assert_eq!(
        envelope.merge_parent_branches,
        vec![BranchId("feature".to_string())]
    );

    let inspection = runtime
        .inspect_what_happened()
        .inspect_commit(merge_commit_id)
        .expect("merge commit inspection");

    assert_eq!(inspection.commit, envelope.commit);
    assert_eq!(
        inspection.changed_records,
        envelope
            .patch
            .authoritative_record_patches
            .iter()
            .map(|record| record.target.clone())
            .collect::<Vec<_>>()
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
