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
            .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
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
    let after = decode(&after);
    let before = &before.directory.as_ref().unwrap().manifest;
    let after = &after.directory.as_ref().unwrap().manifest;
    // D7 retains abandoned identity frontiers. Only identity metadata changes;
    // neither dictionary index may acquire an unpublished meaning.
    assert!(after.symbol_identity_frontier > before.symbol_identity_frontier);
    assert_eq!(before.symbols, after.symbols);
    assert_eq!(before.symbol_buckets, after.symbol_buckets);
    assert_eq!(before.commits, after.commits);

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
