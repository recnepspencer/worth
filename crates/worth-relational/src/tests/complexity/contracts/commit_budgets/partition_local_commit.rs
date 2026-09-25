use super::*;

#[test]
fn complexity_budget_partition_local_commit_reports_touched_partitions() {
    let runtime = runtime_with_test_schema();
    let left = create_entity_in_partition(&runtime, "left", PartitionId(7));
    let right = create_entity_in_partition(&runtime, "right", PartitionId(11));

    runtime.performance_access().reset_counters();
    let _ = update_entity(&runtime, left, "left-updated");
    let single_partition = runtime.performance_access().counters();
    assert_eq!(single_partition.partitions_touched_by_commit, 1);
    assert_eq!(single_partition.full_state_clones, 0);

    runtime.performance_access().reset_counters();
    let _ = create_relation_in_partition(&runtime, left, right, "cross", PartitionId(13));
    let cross_partition = runtime.performance_access().counters();
    assert_eq!(cross_partition.partitions_touched_by_commit, 3);
    assert_eq!(cross_partition.full_state_clones, 0);
}

#[test]
fn complexity_budget_commit_topology_inference_distinguishes_flat_and_graph_mutations() {
    let runtime = runtime_with_test_schema();
    let left = create_entity_in_partition(&runtime, "left", PartitionId(7));
    let right = create_entity_in_partition(&runtime, "right", PartitionId(11));

    runtime.performance_access().reset_counters();
    let _ = update_entity(&runtime, left, "left-updated");
    let flat = runtime.performance_access().counters();
    assert_eq!(
        flat.commit_topology_flags,
        CommitTopology::FlatEntityBatch.mask()
    );

    runtime.performance_access().reset_counters();
    let _ = create_relation_in_partition(&runtime, left, right, "cross", PartitionId(13));
    let graph = runtime.performance_access().counters();
    assert_eq!(
        graph.commit_topology_flags,
        CommitTopology::GraphMutation.mask()
    );
}

#[test]
fn complexity_budget_bulk_create_reserves_exact_logical_slots() {
    let runtime = runtime_with_test_schema();
    runtime.performance_access().reset_counters();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("bulk-entities").push(MutationIntent::Create(
            CreateIntent::BulkEntities(BulkEntityCreateIntent {
                partition_id: PartitionId(41),
                kind_id: KindId(1),
                client_keys: vec![
                    crate::symbols::data::ClientKey::raw("a"),
                    crate::symbols::data::ClientKey::raw("b"),
                    crate::symbols::data::ClientKey::raw("c"),
                ],
                field_patches: vec![
                    crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "a",
                    ),
                    crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "b",
                    ),
                    crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "c",
                    ),
                ],
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let _ = txn.commit(&runtime).unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.bulk_entity_slots_reserved, 3);
    assert_eq!(counters.bulk_relation_slots_reserved, 0);
}

#[test]
fn complexity_budget_mutation_structural_invariants_are_touched_slot_bounded() {
    let runtime = runtime_with_test_schema();
    let target = create_entity(&runtime, "target");
    for index in 0..8 {
        let _ = create_entity(&runtime, &format!("e{index}"));
    }

    runtime.performance_access().reset_counters();
    let _ = update_entity(&runtime, target, "target-updated");
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.entity_slots_touched_by_commit, 1);
    assert_eq!(counters.relation_slots_touched_by_commit, 0);
    assert_eq!(counters.invariant_entity_slot_scans, 1);
}

#[test]
fn complexity_budget_relation_structural_invariants_are_touched_slot_bounded() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");

    runtime.performance_access().reset_counters();
    let _ = create_relation(&runtime, source, target, "r0");
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.entity_slots_touched_by_commit, 0);
    assert_eq!(counters.relation_slots_touched_by_commit, 1);
    assert_eq!(counters.invariant_relation_slot_scans, 1);
}

#[test]
fn complexity_contract_partition_edition_acquisition_is_declared_and_measured() {
    let runtime = runtime_with_test_schema();
    for index in 0..8 {
        let _ = create_entity(&runtime, &format!("e{index}"));
    }

    runtime.performance_access().reset_counters();
    let entity = create_entity(&runtime, "target");
    runtime.performance_access().reset_counters();
    let _ = update_entity(&runtime, entity, "target-updated");
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.full_state_clones, 0);
    assert_eq!(counters.partitions_cloned, 0);
    assert_eq!(counters.entity_slots_cloned, 0);
    assert_eq!(counters.relation_slots_cloned, 0);
    // Neither copy lane fires: an ordinary write owns the spine outright, so
    // there is no edition to fork away from and nothing to reconstruct.
    assert_eq!(counters.reconstructive_state_clones, 0);
    assert_eq!(counters.reconstructive_partitions_materialized, 0);
}
