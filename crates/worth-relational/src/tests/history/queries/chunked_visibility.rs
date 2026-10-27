use crate::tests::support::*;

#[test]
fn chunked_storage_summary_tracks_visibility_boundaries() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_and_chunks(2, 2);
    let first = create_entity_outcome(execution, &runtime, "e0");
    let entity_a = changed_entities(&first)[0];
    let _second = create_entity_outcome(execution, &runtime, "e1");
    let snapshot = runtime.visibility_authority().snapshot();
    let _third = create_entity_outcome(execution, &runtime, "e2");
    let _update = update_entity(execution, &runtime, entity_a, "e0-updated");

    let summary_before_update = runtime
        .storage_access()
        .chunked_storage_summary(execution, snapshot.version_id)
        .unwrap();
    let summary_current = runtime
        .storage_access()
        .chunked_storage_summary(
            execution,
            runtime.history().latest_commit().unwrap().version_id,
        )
        .unwrap();

    assert_eq!(summary_before_update.entity_chunks.len(), 2);
    assert_eq!(summary_before_update.entity_chunks[0].visible_records, 2);
    assert_eq!(summary_before_update.entity_chunks[1].visible_records, 0);
    assert_eq!(summary_current.entity_chunks[1].visible_records, 1);
    assert_eq!(summary_current.entity_chunks[0].slot_len, 2);
}

#[test]
fn chunk_diagnostics_and_packet_plans_are_public_and_stable() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_and_chunks(2, 2);
    let first = create_entity_outcome(execution, &runtime, "e0");
    let second = create_entity_outcome(execution, &runtime, "e1");
    let entity_a = changed_entities(&first)[0];
    let entity_b = changed_entities(&second)[0];
    let snapshot = runtime.visibility_authority().snapshot();
    let packet = explicit_query_packet(
        &runtime,
        &snapshot,
        "pair",
        vec![RecordRef::Entity(entity_a), RecordRef::Entity(entity_b)],
    );

    let plan = runtime
        .storage_access()
        .plan_read_explicit_query_packet(&snapshot, &packet)
        .unwrap();
    let diagnostics = runtime
        .storage_access()
        .chunk_diagnostics(execution, snapshot.version_id)
        .unwrap();

    assert_eq!(plan.target_count, 2);
    assert_eq!(plan.entity_chunk_indexes, vec![0]);
    assert_eq!(diagnostics.entity_chunks_total, 1);
    assert_eq!(diagnostics.entity_chunks_with_visible_records, 1);
}
