use super::*;

#[test]
fn bounded_adjacency_stops_before_allocating_the_unbounded_fanout() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    for ordinal in 0..64 {
        let target = create_entity(execution, &runtime, &format!("target-{ordinal}"));
        let _ = create_relation(
            execution,
            &runtime,
            source,
            target,
            &format!("relation-{ordinal}"),
        );
    }
    let relation_kind = crate::facade::identity::KindId(2);
    let version = runtime.current_version_id();

    runtime.performance_access().reset_counters();
    let limit = runtime
        .read_truth()
        .bounded_outgoing_relations_of_kind_at_version(execution, source, relation_kind, version, 5)
        .expect("read request remains live")
        .expect_err("five units cannot materialize a sixty-four-edge adjacency");

    assert_eq!(limit.consumed_work_units(), 5);
    assert_eq!(limit.relation_records_examined(), 3);
    assert_eq!(limit.endpoint_records_reserved(), 2);

    // The refusal is not merely early: the sixty-four-edge fanout was leased in
    // place and never copied, and the whole traversal ran against one edition.
    let refused = runtime.performance_access().counters();
    assert_eq!(refused.adjacency_kind_slices_leased, 1);
    assert_eq!(refused.adjacency_relation_ids_copied, 0);
    assert_eq!(refused.partition_editions_acquired, 1);
    assert_eq!(refused.full_state_clones, 0);

    let complete = runtime
        .read_truth()
        .bounded_outgoing_relations_of_kind_at_version(
            execution,
            source,
            relation_kind,
            version,
            128,
        )
        .expect("read request remains live")
        .expect("two work units per matching edge admit the exact fanout");
    assert_eq!(complete.relation_records_examined(), 64);
    assert_eq!(complete.endpoint_records_reserved(), 64);
    assert_eq!(complete.work_units(), 128);
    assert_eq!(complete.into_records().len(), 64);

    // The completing read leases exactly as the refused one does; admitting the
    // whole fanout buys no extra substrate copy.
    let completed = runtime.performance_access().counters();
    assert_eq!(completed.adjacency_kind_slices_leased, 2);
    assert_eq!(completed.adjacency_relation_ids_copied, 0);
    assert_eq!(completed.partition_editions_acquired, 2);
}
