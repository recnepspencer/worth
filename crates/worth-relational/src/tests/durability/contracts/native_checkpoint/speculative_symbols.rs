use super::*;

#[test]
fn unpublished_client_keys_leave_checkpoint_dictionary_equal_and_published_keys_restore() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "published-before-abandonment");
    let capture = || {
        runtime
            .durability_authority()
            .native_checkpoint(
                execution,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    };
    let before = capture();
    let mut candidate = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    candidate
        .push_batch(
            batch_create("unpublished-client-key"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, candidate)
        .unwrap();
    // Preparation retains the key in candidate custody until durable publication.
    assert!(!runtime
        .services
        .symbols
        .with_read(|symbols| symbols.contains("unpublished-client-key")));
    drop(candidate);
    let after = capture();
    let decode = |checkpoint: &crate::durability::data::RelationalNativeCheckpoint| {
        crate::durability::log::native_checkpoint_codec::decode_checkpoint(
            execution,
            checkpoint.bytes(),
        )
        .unwrap()
        .checkpoint
    };
    let before = decode(&before);
    let mut after = decode(&after);
    // D7 retains abandoned identity frontiers. Decoding authenticates both
    // dictionary indices; their expanded meaning and commits stay unchanged.
    assert!(after.symbol_identity_frontier > before.symbol_identity_frontier);
    assert_eq!(before.symbol_table, after.symbol_table);
    assert_eq!(before.envelopes, after.envelopes);
    assert_eq!(before.partition_images, after.partition_images);
    assert_eq!(before.aspect_contracts, after.aspect_contracts);
    assert_eq!(
        before.branch_root_schema_images,
        after.branch_root_schema_images
    );
    assert_eq!(before.index_definitions, after.index_definitions);
    assert_eq!(before.lineage, after.lineage);
    // Only D7's burned symbol and record identities may differ. Full equality
    // also protects root images, coverage, schemas, and derived artifacts.
    after.symbol_identity_frontier = before.symbol_identity_frontier;
    after.record_identity.generation_high_water =
        before.record_identity.generation_high_water.clone();
    after.record_identity.reusable_slots = before.record_identity.reusable_slots.clone();
    after.record_identity.append_frontiers = before.record_identity.append_frontiers.clone();
    assert_eq!(before, after);

    let committed = create_entity_outcome(execution, &runtime, "published-after-abandonment");
    let published = capture();
    let mut recovered = durable_runtime_with_test_schema();
    bind_durable_medium(&mut recovered, attached_test_medium(&runtime).attachment());
    let restored = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &published)
        .unwrap();
    assert_eq!(restored.latest_commit, Some(committed.commit.clone()));
    recovered
        .services
        .symbols
        .demand_text(execution, "published-after-abandonment")
        .unwrap();
    recovered
        .services
        .symbols
        .demand_text(execution, "unpublished-client-key")
        .unwrap();
    assert!(recovered
        .services
        .symbols
        .with_read(|symbols| symbols.contains("published-after-abandonment")));
    assert!(!recovered
        .services
        .symbols
        .with_read(|symbols| symbols.contains("unpublished-client-key")));
    create_entity_outcome(execution, &recovered, "editable-after-dictionary-recovery");
}
