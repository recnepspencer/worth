use super::fixtures::*;

#[test]
fn publication_snapshot_handle_reads_without_becoming_a_pinned_snapshot() {
    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(&runtime, "first");

    let retention = runtime.retention().inspect_plan();
    let read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .unwrap();
    let inspection = runtime
        .read_truth()
        .inspect_snapshot(&outcome.snapshot)
        .unwrap();
    let packet = explicit_query_packet(
        &runtime,
        &outcome.snapshot,
        "entities",
        vec![RecordRef::Entity(changed_entities(&outcome)[0])],
    );

    assert_eq!(retention.active_snapshot_count, 0);
    assert_eq!(retention.snapshot_pinned_entities, 0);
    assert_eq!(retention.snapshot_pinned_relations, 0);
    assert_eq!(read.entities.len(), 1);
    assert_eq!(inspection.pinned_entity_count, 0);
    assert_eq!(inspection.entity_count, 1);
    assert!(runtime
        .storage_access()
        .plan_read_explicit_query_packet(&outcome.snapshot, &packet)
        .is_some());
    assert_eq!(
        execute_explicit_query(
            &runtime,
            &outcome.snapshot,
            "entities",
            vec![RecordRef::Entity(changed_entities(&outcome)[0])],
        )
        .result
        .entities
        .len(),
        1
    );
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&outcome.snapshot)
        .is_ok());
    assert!(runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .is_none());
}

#[test]
fn publication_snapshot_reads_use_authoritative_published_binding_version() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "first");
    let updated = update_entity(&runtime, changed_entities(&created)[0], "second");
    let mut stale_handle = updated.snapshot.clone();
    stale_handle.version_id = created.snapshot.version_id;

    let read = runtime.read_truth().read_snapshot(&stale_handle).unwrap();
    let inspection = runtime
        .read_truth()
        .inspect_snapshot(&stale_handle)
        .unwrap();

    assert_eq!(read.snapshot.version_id, updated.snapshot.version_id);
    assert_eq!(inspection.version_id, updated.snapshot.version_id);
    assert_eq!(read.entities.len(), 1);
    assert_eq!(
        read_entity_field(&read.entities[0], field_key("name")),
        Some("second".into())
    );
}

#[test]
fn released_publication_handles_stop_counting_as_readable_runtime_state() {
    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(&runtime, "first");
    let second = create_entity_outcome(&runtime, "second");

    let before = runtime.storage_access().storage_stats();
    assert_eq!(before.published_snapshot_handle_count, 2);

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&first.snapshot)
        .is_ok());
    let after_first_release = runtime.storage_access().storage_stats();
    assert_eq!(after_first_release.published_snapshot_handle_count, 1);
    assert!(runtime
        .read_truth()
        .read_snapshot(&first.snapshot)
        .is_none());
    assert!(runtime
        .read_truth()
        .read_snapshot(&second.snapshot)
        .is_some());

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&second.snapshot)
        .is_ok());
    let after_second_release = runtime.storage_access().storage_stats();
    assert_eq!(after_second_release.published_snapshot_handle_count, 0);
}

#[test]
fn publication_handle_retention_is_bounded_by_policy() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4096,
            max_published_snapshot_handles: 2,
            max_active_snapshot_handles: 4_096,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 8,
            candidate_max_lifetime_millis: 30_000,
            max_prepared_root_bytes: 268_435_456,
        })
        .build();
    let first = create_entity_outcome(&runtime, "first");
    let second = create_entity_outcome(&runtime, "second");
    let before = crate::tests::support::test_owner_main_basis(&runtime).unwrap();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("third"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        transaction.commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        ),
        Err(
            crate::transactions::data::TransactionCommitError::PublicationDeferred {
                deferred:
                    crate::mvcc::RelationalPublicationDeferred::PublishedSnapshotCapacityExhausted {
                        maximum_handles: 2,
                    },
                ..
            }
        )
    ));

    let stats = runtime.storage_access().storage_stats();

    assert_eq!(stats.published_snapshot_handle_count, 2);
    assert!(runtime
        .read_truth()
        .read_snapshot(&first.snapshot)
        .is_some());
    assert!(runtime
        .read_truth()
        .read_snapshot(&second.snapshot)
        .is_some());
    assert_eq!(
        crate::tests::support::test_owner_main_basis(&runtime)
            .unwrap()
            .descriptor(),
        before.descriptor()
    );
}

#[test]
fn published_handle_and_admitted_observation_remain_exact_until_release() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4096,
            max_published_snapshot_handles: 4,
            max_active_snapshot_handles: 4_096,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 8,
            candidate_max_lifetime_millis: 30_000,
            max_prepared_root_bytes: 268_435_456,
        })
        .build();
    let first = create_entity_outcome(&runtime, "first");
    let identity = runtime.main_branch_identity();
    let (_, first_basis) = runtime.observe_branch(&identity).unwrap();
    let first_observation = first_basis.observation();
    let entity = changed_entities(&first)[0];
    update_entity(&runtime, entity, "second");
    update_entity(&runtime, entity, "third");
    update_entity(&runtime, entity, "fourth");

    assert!(runtime
        .read_truth()
        .read_snapshot(&first.snapshot)
        .is_some());
    let observed = runtime
        .snapshots()
        .snapshot_for_observation(&first_observation)
        .expect("admitted observation carries its selected root independently");
    assert_eq!(observed.version_id, first.version_id);
    assert!(runtime.read_truth().read_snapshot(&observed).is_some());
}

#[test]
fn leased_commit_preparation_preserves_publication_surfaces() {
    let serial = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4096,
            max_published_snapshot_handles: 3,
            max_active_snapshot_handles: 4_096,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 8,
            candidate_max_lifetime_millis: 30_000,
            max_prepared_root_bytes: 268_435_456,
        })
        .build();
    let parallel = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4096,
            max_published_snapshot_handles: 3,
            max_active_snapshot_handles: 4_096,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 8,
            candidate_max_lifetime_millis: 30_000,
            max_prepared_root_bytes: 268_435_456,
        })
        .build();

    let _ = create_entity_outcome(&serial, "first");
    let _serial_second = create_entity_outcome(&serial, "second");
    let _serial_third = create_entity_outcome(&serial, "third");

    parallel.performance_access().reset_counters();
    let lease = test_execution_lease();
    let _ = create_entity_outcome_with_lease(&parallel, "first", &lease);
    let parallel_second = create_entity_outcome_with_lease(&parallel, "second", &lease);
    let parallel_third = create_entity_outcome_with_lease(&parallel, "third", &lease);

    let serial_bundle = serial.publication().latest_bundle().unwrap().clone();
    let parallel_bundle = parallel.publication().latest_bundle().unwrap().clone();
    let parallel_stats = parallel.storage_access().storage_stats();
    let diagnostics = parallel.publication().diagnostics();

    assert_eq!(parallel_bundle.commit, serial_bundle.commit);
    assert_eq!(parallel_bundle.patch, serial_bundle.patch);
    assert_eq!(parallel_bundle.replay, serial_bundle.replay);
    assert_eq!(parallel_bundle.snapshot, parallel_third.snapshot);
    assert_eq!(parallel_stats.published_snapshot_handle_count, 3);
    assert!(parallel
        .read_truth()
        .read_snapshot(&parallel_second.snapshot)
        .is_some());
    assert!(parallel
        .read_truth()
        .read_snapshot(&parallel_third.snapshot)
        .is_some());
    assert!(diagnostics
        .minimal_summaries()
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::CommitPublished));
    assert_eq!(
        parallel
            .performance_access()
            .counters()
            .post_commit_consumer_packet_count,
        3
    );
    assert_eq!(
        parallel
            .performance_access()
            .counters()
            .post_commit_serial_strategy_count,
        3
    );
    assert_eq!(
        parallel
            .performance_access()
            .counters()
            .post_commit_parallel_strategy_count,
        0
    );
}
