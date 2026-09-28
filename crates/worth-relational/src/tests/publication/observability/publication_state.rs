use super::fixtures::*;

#[test]
fn diagnostics_and_replay_are_emitted_for_commit() {
    let runtime = runtime_with_test_schema();
    let _entity = create_entity(&runtime, "first");
    let diagnostics = runtime.publication().diagnostics();

    assert!(diagnostics.artifacts().iter().any(|artifact| {
        artifact.scope == DiagnosticsScope::Transaction
            && artifact.kind == DiagnosticsArtifactKind::MinimalSummary
    }));
    assert!(diagnostics
        .minimal_summaries()
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::EntityCreated));
    assert!(runtime.publication().latest_patch().is_some());
    assert!(runtime.publication().latest_replay().is_some());
    assert_eq!(
        runtime
            .publication()
            .latest_replay()
            .unwrap()
            .schema_authority,
        test_schema_registry().authority_snapshot()
    );
    assert_eq!(
        runtime.publication().diagnostic_artifact_count(),
        diagnostics.artifacts().len()
    );
}

#[test]
fn publication_diagnostics_since_fail_closes_for_stale_cursor() {
    let runtime = runtime_with_test_schema();
    let _entity = create_entity(&runtime, "first");
    let artifact_count = runtime.publication().diagnostic_artifact_count();

    assert!(runtime
        .publication()
        .diagnostics_since(artifact_count + 100)
        .is_empty());
}

#[test]
fn publication_observation_snapshot_tracks_latest_publication_state() {
    let runtime = runtime_with_test_schema();

    let empty = runtime.publication().observation_snapshot();
    assert_eq!(empty.latest_commit_id, None);
    assert_eq!(empty.publication_snapshot_id, None);
    assert_eq!(empty.publication_status, None);
    assert_eq!(empty.latest_patch_position, None);
    assert!(!empty.latest_patch_present);
    assert!(!empty.latest_replay_present);
    assert_eq!(empty.diagnostics_artifact_count, 0);

    let created = create_entity_outcome(&runtime, "first");
    let observed = runtime.publication().observation_snapshot();
    let publication = runtime.publication();
    let bundle = publication.latest_bundle().unwrap();

    assert_eq!(observed.latest_commit_id, Some(created.commit.commit_id));
    assert_eq!(
        observed.publication_snapshot_id,
        Some(bundle.snapshot.snapshot_id)
    );
    assert_eq!(observed.publication_status, Some(bundle.status.clone()));
    assert_eq!(observed.latest_patch_position, Some(bundle.patch.position));
    assert_eq!(
        observed.latest_patch_record_count,
        Some(bundle.patch.authoritative_record_patches.len())
    );
    assert_eq!(
        observed.latest_replay_commit_id,
        Some(bundle.replay.commit_id)
    );
    assert!(observed.latest_patch_present);
    assert!(observed.latest_replay_present);
    assert!(observed.diagnostics_artifact_count > 0);
}

#[test]
fn publication_artifact_snapshot_tracks_latest_patch_and_replay_with_observation() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "first");

    let snapshot = runtime.publication().artifact_snapshot();
    let publication = runtime.publication();
    let bundle = publication.latest_bundle().unwrap();

    assert_eq!(
        snapshot.observation.latest_commit_id,
        Some(created.commit.commit_id)
    );
    assert_eq!(snapshot.latest_patch, Some(bundle.patch.clone()));
    assert_eq!(snapshot.latest_replay, Some(bundle.replay.clone()));
}

#[test]
fn publication_diagnostics_snapshot_tracks_observation_and_artifacts_together() {
    let runtime = runtime_with_test_schema();
    let _created = create_entity_outcome(&runtime, "first");

    let snapshot = runtime.publication().diagnostics_snapshot();
    let publication = runtime.publication();

    assert_eq!(snapshot.observation, publication.observation_snapshot());
    assert_eq!(snapshot.diagnostics, publication.diagnostic_artifacts());
}

#[test]
fn publication_bundle_is_the_single_visible_commit_surface() {
    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(&runtime, "first");
    let publication = runtime.publication();
    let bundle = publication.latest_bundle().unwrap();

    assert_eq!(outcome.publication_status, PublicationStatus::Published);
    assert_eq!(bundle.snapshot, outcome.snapshot);
    assert_eq!(bundle.commit, outcome.commit);
    assert_eq!(bundle.commit, runtime.history().latest_commit().unwrap());
    assert_eq!(bundle.patch, runtime.publication().latest_patch().unwrap());
    assert_eq!(
        bundle.replay,
        runtime.publication().latest_replay().unwrap()
    );
}

#[test]
fn snapshot_audit_failure_blocks_publication() {
    let runtime = runtime_with_declared_aspect_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::snapshot_publication_blocking(
            InvariantRule::MaxSnapshotEntities(0),
        )],
    });
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(batch_create("blocked"))
        .expect("test staging stays within configured resource budgets");
    let error = txn.commit(&runtime).unwrap_err();

    assert!(matches!(
        error,
        TransactionCommitError::Publication { error: ref publication, .. }
            if publication.stage == PublicationStage::InvariantCheck
    ));
    assert!(runtime.publication().latest_bundle().is_none());
}

#[test]
fn bulk_packets_are_the_primary_read_surface() {
    let runtime = runtime_with_test_schema();
    let entity = create_entity(&runtime, "first");
    let snapshot = runtime.visibility_authority().snapshot();
    let plan = runtime
        .storage_access()
        .plan_read_explicit_query_packet(
            &snapshot,
            &explicit_query_packet(
                &runtime,
                &snapshot,
                "entities",
                vec![RecordRef::Entity(entity)],
            ),
        )
        .unwrap();
    let result = execute_explicit_query(
        &runtime,
        &snapshot,
        "entities",
        vec![RecordRef::Entity(entity)],
    )
    .result;

    assert_eq!(plan.entity_chunk_indexes, vec![0]);
    assert_eq!(result.entities.len(), 1);
}
