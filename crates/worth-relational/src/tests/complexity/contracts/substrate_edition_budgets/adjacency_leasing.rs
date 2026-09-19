use crate::tests::support::*;

const RELATION_KIND: crate::facade::identity::KindId = crate::facade::identity::KindId(2);

#[test]
fn complexity_budget_bounded_adjacency_leases_its_fanout_without_copying_it() {
    let mut runtime = runtime_with_test_schema();
    let anchor = fanned_out_anchor(&mut runtime, 64);
    let version_id = runtime.current_version_id();

    runtime.performance_access().reset_counters();
    let refusal = runtime
        .read_truth()
        .bounded_outgoing_relations_of_kind_at_version(anchor, RELATION_KIND, version_id, 6)
        .expect_err("six work units cannot admit a sixty-four edge fanout");
    let counters = runtime.performance_access().counters();

    // The bound is applied to the substrate's own storage, so a refusal at six
    // units never touched the other fifty-eight edges.
    assert_eq!(refusal.consumed_work_units(), 6);
    assert_eq!(counters.adjacency_kind_slices_leased, 1);
    assert_eq!(counters.adjacency_relation_ids_copied, 0);
}

#[test]
fn complexity_budget_bounded_adjacency_acquires_one_edition_for_the_whole_traversal() {
    let mut runtime = runtime_with_test_schema();
    let anchor = fanned_out_anchor(&mut runtime, 32);
    let version_id = runtime.current_version_id();

    runtime.performance_access().reset_counters();
    let records = runtime.read_truth().outgoing_relations_of_kind_at_version(
        anchor,
        RELATION_KIND,
        version_id,
    );
    let counters = runtime.performance_access().counters();

    // Thirty-two resolved records against exactly one pinned edition. A
    // per-record acquisition would make this thirty-two.
    assert_eq!(records.len(), 32);
    assert_eq!(counters.partition_editions_acquired, 1);
    assert_eq!(counters.adjacency_relation_ids_copied, 0);
}

#[test]
fn complexity_budget_whole_neighborhood_read_charges_every_id_it_copies() {
    let mut runtime = runtime_with_test_schema();
    let anchor = fanned_out_anchor(&mut runtime, 16);
    let version_id = runtime.current_version_id();

    runtime.performance_access().reset_counters();
    let relations = runtime
        .storage_access()
        .outgoing_relations_for_entity(anchor, version_id);
    let counters = runtime.performance_access().counters();

    // The negative twin of the leasing readers: this caller asked for every id,
    // so the copy is real and is charged rather than laundered into the zero
    // the bounded readers legitimately report.
    assert_eq!(relations.len(), 16);
    assert_eq!(counters.adjacency_relation_ids_copied, 16);
    assert_eq!(counters.adjacency_kind_slices_leased, 0);
    assert_eq!(counters.partition_editions_acquired, 1);
}

#[test]
fn complexity_budget_frontier_adjacency_acquires_one_edition_for_the_whole_frontier() {
    let mut runtime = runtime_with_test_schema();
    let anchors = (0..8)
        .map(|ordinal| fanned_out_anchor_named(&mut runtime, ordinal, 4))
        .collect::<std::collections::BTreeSet<_>>();
    let version_id = runtime.current_version_id();

    runtime.performance_access().reset_counters();
    let frontier = runtime
        .read_truth()
        .bounded_outgoing_relations_for_frontier_at_version(
            &anchors,
            RELATION_KIND,
            version_id,
            usize::MAX,
        )
        .expect("an unbounded frontier read cannot exhaust usize::MAX work");
    let counters = runtime.performance_access().counters();

    assert_eq!(frontier.adjacency_lists_read(), 8);
    assert_eq!(frontier.endpoint_records_reserved(), 32);
    // One edition for a width-eight frontier, and one lease per frontier
    // entity, never a copy of any entity's fanout.
    assert_eq!(counters.partition_editions_acquired, 1);
    assert_eq!(counters.adjacency_kind_slices_leased, 8);
    assert_eq!(counters.adjacency_relation_ids_copied, 0);
}

#[test]
fn projected_frontier_adjacency_uses_the_selected_branch_root() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "fork-source");
    let child = create_branch_from_main(&runtime, "projected-adjacency-child");
    let source = create_entity_in_partition_on_branch(
        &runtime,
        "child-source",
        crate::facade::identity::PartitionId::main(),
        child.clone(),
    );
    let target = create_entity_in_partition_on_branch(
        &runtime,
        "child-target",
        crate::facade::identity::PartitionId::main(),
        child.clone(),
    );
    create_relation_in_partition_on_branch(
        &runtime,
        source,
        target,
        "child-edge",
        "child-edge",
        crate::facade::identity::PartitionId::main(),
        child.clone(),
    );
    let snapshot = snapshot_for_owner_branch(&runtime, &child);
    let projection = runtime.read_truth().project_snapshot(&snapshot).unwrap();
    let records = projection
        .bounded_outgoing_relations_for_frontier(
            &std::collections::BTreeSet::from([source]),
            RELATION_KIND,
            8,
        )
        .unwrap()
        .into_records();
    assert_eq!(records.len(), 1);
    assert_eq!((records[0].source, records[0].target), (source, target));
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
}

fn fanned_out_anchor(
    runtime: &mut crate::runtime::RelationalRuntime,
    degree: usize,
) -> crate::facade::identity::EntityId {
    fanned_out_anchor_named(runtime, 0, degree)
}

fn fanned_out_anchor_named(
    runtime: &mut crate::runtime::RelationalRuntime,
    ordinal: usize,
    degree: usize,
) -> crate::facade::identity::EntityId {
    let anchor = create_entity(runtime, &format!("anchor-{ordinal:04}"));
    for edge in 0..degree {
        let target = create_entity(runtime, &format!("target-{ordinal:04}-{edge:04}"));
        let _ = create_relation(
            runtime,
            anchor,
            target,
            &format!("edge-{ordinal:04}-{edge:04}"),
        );
    }
    anchor
}
