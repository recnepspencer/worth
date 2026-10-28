use crate::tests::support::*;

#[test]
fn live_history_trimming_runs_only_in_the_declared_full_retention_pass() {
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

    // Untouched records are deliberately part of Full maintenance's footprint.
    for index in 0..16 {
        let outcome = create_entity_outcome(execution, &runtime, &format!("untouched-{index}"));
        release_test_commit_snapshot(&runtime, &outcome);
    }
    runtime.performance_access().reset_counters();
    let update_a1 = update_entity(execution, &runtime, entity_a, "a-1");
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&update_a1.snapshot)
        .is_ok());
    let _ = update_entity(execution, &runtime, entity_a, "a-2");
    assert!(
        runtime.entity_history_len_for_test(entity_a) > 1,
        "D9 Head publication must not perform implicit Full history maintenance"
    );
    let publication = runtime.performance_access().counters();
    assert_eq!(publication.live_history_slots_visited, 0);
    let history_before = runtime.entity_history_len_for_test(entity_a);
    runtime.performance_access().reset_counters();
    runtime.retention().run_pass(execution).unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(runtime.entity_history_len_for_test(entity_a), 1);
    assert_eq!(runtime.entity_history_len_for_test(entity_b), 1);
    assert!(counters.live_entity_history_entries_trimmed >= 1);
    assert!(
        (18..36).contains(&counters.live_history_slots_visited),
        "{counters:?}"
    );
    assert!(
        (18..(4 * (18 + history_before))).contains(&counters.live_history_entries_visited),
        "{counters:?}"
    );
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
        .outgoing_relations_for_entity(execution, source, version_id)
        .expect("funded adjacency read succeeds");
    let incoming = runtime
        .storage_access()
        .incoming_relations_for_entity(execution, target, version_id)
        .expect("funded adjacency read succeeds");
    let counters = runtime.performance_access().counters();

    assert_eq!(outgoing, vec![relation]);
    assert_eq!(incoming, vec![relation]);
    assert_eq!(counters.visibility_relation_slot_scans, 0);
}
