use crate::tests::support::*;

#[test]
fn complexity_budget_ordinary_commit_copies_neither_lane_of_the_substrate() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "anchor");

    runtime.performance_access().reset_counters();
    let _ = update_entity(execution, &runtime, entity, "anchor-updated");
    let counters = runtime.performance_access().counters();

    // Nothing observes the substrate, so the writer mutates in place.
    assert_eq!(counters.full_state_clones, 0);
    assert_eq!(counters.reconstructive_state_clones, 0);
    assert_eq!(counters.reconstructive_partitions_materialized, 0);
}

#[test]
fn complexity_budget_write_under_an_outstanding_reader_copies_the_spine_once() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "anchor");

    // The positive twin: an edition held across a write is exactly the
    // condition that forces an honest copy-on-write of the map spine, and it
    // is charged to the ordinary lane rather than relabelled.
    let observer = runtime.acquire_partition_edition();
    runtime.performance_access().reset_counters();
    let _ = update_entity(execution, &runtime, entity, "anchor-updated");
    let counters = runtime.performance_access().counters();
    drop(observer);

    assert!(
        counters.full_state_clones >= 1,
        "an observed substrate must charge its copy: {counters:?}"
    );
    assert_eq!(counters.reconstructive_state_clones, 0);
}

#[test]
fn complexity_budget_forking_a_runtime_charges_the_reconstructive_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _ = create_entity_in_partition(execution, &runtime, "left", PartitionId(7));
    let _ = create_entity_in_partition(execution, &runtime, "right", PartitionId(11));

    runtime.performance_access().reset_counters();
    let forked = runtime.fork().expect("a test runtime can fork");
    let counters = runtime.performance_access().counters();
    drop(forked);

    // Whole-state materialization is legitimate here and must land in its own
    // lane, so the ordinary-lane zero above stays interpretable.
    assert_eq!(counters.full_state_clones, 0);
    assert_eq!(counters.reconstructive_state_clones, 1);
    assert!(
        counters.reconstructive_partitions_materialized >= 2,
        "a fork materializes every partition it detaches: {counters:?}"
    );
    // The partition copies a fork performs belong to its own lane. If they
    // leaked into the ordinary counters, an ordinary-lane zero elsewhere would
    // stop meaning anything.
    assert_eq!(counters.ordinary_partitions_copied_on_write, 0);
    assert_eq!(counters.ordinary_partition_slots_copied_on_write, 0);
}

#[test]
fn complexity_budget_read_only_retention_sweep_pins_one_edition_for_every_slot() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    for ordinal in 0..8 {
        let _ = create_entity(execution, &runtime, &format!("slot-{ordinal:04}"));
    }

    runtime.performance_access().reset_counters();
    let summary = runtime
        .inspect_what_happened()
        .retention_summary(
            execution,
            &crate::facade::inspection::RetentionInspectionRequest {
                max_entity_slots_scanned: 64,
                max_relation_slots_scanned: 64,
                max_work_units: 256,
            },
        )
        .unwrap();
    let counters = runtime.performance_access().counters();

    // A read-only sweep over S slots acquires the substrate once, not once per
    // slot surface. The read-modify-read retention pass deliberately does not
    // pin, because pinning across its writes would force partition copies.
    assert_eq!(
        summary.availability,
        crate::facade::inspection::InspectionAvailability::Direct
    );
    assert_eq!(counters.partition_editions_acquired, 1);
}
