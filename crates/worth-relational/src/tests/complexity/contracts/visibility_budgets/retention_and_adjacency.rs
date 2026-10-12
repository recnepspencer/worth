use crate::tests::support::*;

#[test]
fn complexity_budget_live_history_trimming_is_touched_record_bounded() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let create_a = create_entity_outcome(execution, &runtime, "a");
    let entity_a = changed_entities(&create_a)[0];
    let create_b = create_entity_outcome(execution, &runtime, "b");
    let entity_b = changed_entities(&create_b)[0];
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&create_a.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&create_b.snapshot)
        .is_ok());

    runtime.performance_access().reset_counters();
    let update_a1 = update_entity(execution, &runtime, entity_a, "a-1");
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&update_a1.snapshot)
        .is_ok());
    let _ = update_entity(execution, &runtime, entity_a, "a-2");
    let counters = runtime.performance_access().counters();

    assert_eq!(runtime.entity_history_len_for_test(entity_a), 1);
    assert_eq!(runtime.entity_history_len_for_test(entity_b), 1);
    assert!(counters.live_entity_history_entries_trimmed >= 1);
}

#[test]
fn complexity_budget_bidirectional_adjacency_avoids_relation_scans() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let relation = create_relation(execution, &runtime, source, target, "r0");
    let version_id = runtime.history().latest_commit().unwrap().version_id;

    runtime.performance_access().reset_counters();
    let outgoing = runtime
        .storage_access()
        .outgoing_relations_for_entity(source, version_id);
    let incoming = runtime
        .storage_access()
        .incoming_relations_for_entity(target, version_id);
    let counters = runtime.performance_access().counters();

    assert_eq!(outgoing, vec![relation]);
    assert_eq!(incoming, vec![relation]);
    assert_eq!(counters.visibility_relation_slot_scans, 0);
}
