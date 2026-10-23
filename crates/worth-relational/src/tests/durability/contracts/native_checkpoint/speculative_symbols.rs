use super::*;

#[test]
fn unpublished_client_keys_leave_checkpoint_bytes_equal_and_published_keys_restore() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = persisted_runtime_with_test_schema();
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
    candidate
        .merged_plan(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    // Real preparation has interned this key, but has no publication authority.
    assert!(runtime
        .services
        .symbols
        .with_read(|symbols| symbols.contains("unpublished-client-key")));
    drop(candidate);
    assert_eq!(before.bytes(), capture().bytes());

    let committed = create_entity_outcome(execution, &runtime, "published-after-abandonment");
    let published = capture();
    let mut recovered = persisted_runtime_with_test_schema();
    let restored = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &published)
        .unwrap();
    assert_eq!(restored.latest_commit, Some(committed.commit.clone()));
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
