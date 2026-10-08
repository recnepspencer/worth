use super::*;

#[test]
fn complexity_budget_preparation_packetization_is_chunked_for_broad_deltas() {
    let runtime = runtime_with_test_schema();
    runtime.performance_access().reset_counters();

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("bulk-entities").push(MutationIntent::Create(
            CreateIntent::BulkEntities(BulkEntityCreateIntent {
                partition_id: PartitionId(41),
                kind_id: KindId(1),
                client_keys: (0..65)
                    .map(|index| crate::symbols::data::ClientKey::raw(format!("e{index}")))
                    .collect(),
                field_patches: (0..65)
                    .map(|index| {
                        crate::tests::support::aspect_field_patch_from_values([(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            crate::tests::support::string_aspect_value(&format!("e{index}")),
                        )])
                    })
                    .collect(),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = runtime
        .commit_branch_transaction_with_lease(txn, &test_execution_lease())
        .unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(outcome.changed_records.len(), 65);
    assert!(counters.preparation_packet_count <= 8);
    assert!(counters.preparation_packet_item_count >= outcome.changed_records.len());
    assert!(counters.preparation_packet_peak_width_total >= 16);
    assert!(counters.preparation_scope_unit_count >= 1);
    assert!(counters.preparation_staged_parallel_strategy_count >= 1);
    assert!(counters.preparation_packet_count < outcome.changed_records.len());
}

#[test]
fn complexity_budget_preparation_narrow_delta_falls_back_to_serial() {
    let runtime = runtime_with_test_schema();
    runtime.performance_access().reset_counters();

    let _ = create_entity_outcome(&runtime, "narrow");
    let counters = runtime.performance_access().counters();

    assert!(counters.preparation_packet_count <= 3);
    assert!(counters.preparation_packet_item_count >= 1);
    assert!(counters.preparation_packet_peak_width_total >= 1);
    assert!(counters.preparation_scope_unit_count >= 1);
    assert!(counters.preparation_serial_strategy_count >= 1);
}
