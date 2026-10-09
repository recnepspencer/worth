use super::*;

#[test]
fn runtime_recovers_settlement_after_external_capability_is_dropped() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "runtime-recovery-anchor");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("runtime-owned-deferred-settlement"),
            AllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(transaction, AllocationPolicy::SystemAllocation)
        .unwrap();
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("runtime recovery candidate performs before its injected append fault");
    };
    let commit_id = performed.canonical_commit().commit.commit_id;
    runtime.durability.arm_append_failure();
    let error = runtime
        .settle_performed_publication(performed)
        .expect_err("durable append fault returns deferred settlement");
    assert_eq!(runtime.publication_binding().pending_settlement_count(), 1);
    assert_eq!(runtime.visibility.published_snapshot_handle_count(), 1);
    drop(error);
    assert_eq!(runtime.publication_binding().pending_settlement_count(), 1);
    assert_eq!(runtime.visibility.published_snapshot_handle_count(), 1);

    let mut blocked = test_owner_begin_transaction_for_main(&runtime);
    blocked
        .push_batch(
            batch_create("blocked-unsettled-child"),
            AllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(runtime
        .prepare_branch_transaction(blocked, AllocationPolicy::SystemAllocation,)
        .unwrap_err()
        .detail()
        .contains("requires explicit owner settlement"));

    let repaired = runtime
        .repair_pending_publication_settlement(commit_id)
        .expect("runtime-owned recovery survives loss of the external capability");
    assert_eq!(repaired.commit_id, commit_id);
    assert_eq!(runtime.publication_binding().pending_settlement_count(), 0);
    assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0);
    let child = create_entity_outcome(&runtime, "child-after-runtime-recovery");
    assert_eq!(child.commit.parents, vec![commit_id]);
    release_test_commit_snapshot(&runtime, &child);
}
