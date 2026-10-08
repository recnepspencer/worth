use crate::tests::support::*;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

#[test]
fn direct_publication_settles_before_a_child_can_acknowledge() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "settlement-anchor");
    runtime
        .durability_authority()
        .checkpoint()
        .expect("checkpoint seals the baseline before the direct parent");

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("direct-parent"),
            AllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(transaction, AllocationPolicy::SystemAllocation)
        .expect("direct parent prepares");
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("direct parent must perform");
    };
    let parent_id = performed.canonical_commit().commit.commit_id;
    let parent_position = performed.patch_position();

    assert!(runtime
        .history()
        .recent_commit_ids(None, 8)
        .contains(&parent_id));
    assert_eq!(
        runtime
            .publication()
            .read_patch_stream(PatchStreamRequest::default())
            .expect("direct parent is stream-visible")
            .latest_commit_id,
        Some(parent_id)
    );
    let subscriber = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(8))
        .expect("CDC sees the direct parent through canonical stream inventory");
    assert_eq!(subscriber.latest_commit_id, Some(parent_id));
    assert!(subscriber
        .patches
        .iter()
        .any(|patch| patch.position == parent_position));

    let fork_denial = runtime
        .fork()
        .expect_err("runtime fork is denied while direct publication is unsettled");
    assert_eq!(
        fork_denial,
        crate::runtime::RelationalRuntimeForkDenial::PerformedPublicationRequiresSettlement {
            commit_id: parent_id,
        }
    );
    let mut blocked_child = test_owner_begin_transaction_for_main(&runtime);
    blocked_child
        .push_batch(
            batch_create("blocked-before-settlement"),
            AllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let child_denial = runtime
        .prepare_branch_transaction(blocked_child, AllocationPolicy::SystemAllocation)
        .expect_err("a child cannot hide its parent's unfinished settlement");
    assert!(format!("{child_denial:?}").contains("requires explicit owner settlement"));
    let settled_parent = runtime
        .settle_performed_publication(performed)
        .expect("the direct owner explicitly settles its performed publication");
    release_test_commit_snapshot(&runtime, &settled_parent);

    let child = create_entity_outcome(&runtime, "ordinary-child");
    assert_eq!(child.commit.parents, vec![parent_id]);
    let durability = runtime.durability();
    let durable = durability.durable_log();
    let tail = &durable[durable.len().saturating_sub(2)..];
    assert_eq!(tail.len(), 2, "checkpoint tail contains parent then child");
    assert_eq!(tail[0].envelope().commit.commit_id, parent_id);
    assert_eq!(tail[0].position(), parent_position);
    assert_eq!(tail[1].envelope().commit.commit_id, child.commit.commit_id);
    assert!(tail[0].position() < tail[1].position());
    let child_position = tail[1].position();
    release_test_commit_snapshot(&runtime, &child);

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(plan)
        .expect("fresh runtime recovers the baseline checkpoint plus exact tail");
    assert_eq!(
        recovered
            .history()
            .immutable_commit_receipt(parent_id)
            .expect("settled direct parent recovers")
            .commit_id,
        parent_id
    );
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .expect("ordinary child remains the recovered head")
            .commit_id,
        child.commit.commit_id
    );
    let continued = create_entity_outcome(&recovered, "post-recovery-child");
    let continued_position = recovered
        .history
        .canonical_stream_position(continued.commit.commit_id)
        .expect("post-recovery child receives a stream position");
    assert!(child_position < continued_position);
    release_test_commit_snapshot(&recovered, &continued);
}

/// The deferred carrier is a view of a runtime-owned record, so holding it
/// across repair may not keep alive the published snapshot that record opened.
/// Both axes are asserted: the published handle registry and the capacity owner.
#[test]
fn failed_durable_append_returns_an_idempotent_owner_repair_capability() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "repair-anchor");
    runtime
        .durability_authority()
        .checkpoint()
        .expect("repair baseline checkpoint");

    let published_before = runtime.visibility.published_snapshot_handle_count();
    let occupied_before = runtime
        .preparation_runtime_snapshot()
        .published_snapshot_count();

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("performed-before-append-fault"),
            AllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(transaction, AllocationPolicy::SystemAllocation)
        .expect("fault candidate prepares");
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("fault candidate performs");
    };
    let commit_id = performed.canonical_commit().commit.commit_id;
    let position = performed.patch_position();
    runtime.durability.arm_append_failure();
    let error = runtime
        .settle_performed_publication(performed)
        .expect_err("injected append fault defers settlement after performance");
    let settlement = error
        .deferred_settlement()
        .expect("the owner receives exact settlement repair authority");
    assert_eq!(settlement.commit().commit_id, commit_id);
    assert_eq!(settlement.patch_position(), position);
    assert_eq!(
        runtime
            .durability_authority()
            .checkpoint()
            .expect_err("unsettled performed publication blocks checkpoint")
            .class,
        crate::durability::data::RecoveryFailureClass::PerformedPublicationRequiresSettlement
    );

    assert_eq!(
        runtime.visibility.published_snapshot_handle_count(),
        published_before + 1,
        "the deferred lane keeps the published snapshot it already opened"
    );
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        occupied_before + 1,
        "the deferred lane still occupies the capacity slot it reserved"
    );

    let repaired = runtime
        .repair_deferred_publication_settlement(settlement)
        .expect("owner retries the exact missing durable append");
    assert_eq!(repaired.commit_id, commit_id);
    // The carrier is still held here and is still a usable view of the same
    // route, so a carrier that also held the release obligation would surface as
    // a leaked handle and a leaked capacity slot on exactly these two axes.
    assert_eq!(settlement.commit().commit_id, commit_id);
    assert_eq!(
        runtime.visibility.published_snapshot_handle_count(),
        published_before,
        "terminal settlement releases the published snapshot the carrier only names"
    );
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        occupied_before,
        "terminal settlement leaks no published-snapshot capacity to a live carrier"
    );

    let repaired_again = runtime
        .repair_deferred_publication_settlement(settlement)
        .expect("repeating an already successful repair is harmless");
    assert_eq!(repaired_again, repaired);
    assert_eq!(
        runtime.visibility.published_snapshot_handle_count(),
        published_before,
        "a second repair releases nothing a second time"
    );
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        occupied_before,
        "a second repair moves no published-snapshot capacity"
    );
    assert_eq!(
        runtime
            .durability()
            .durable_log()
            .iter()
            .filter(|entry| entry.envelope().commit.commit_id == commit_id)
            .count(),
        1
    );
    runtime
        .durability_authority()
        .checkpoint()
        .expect("checkpoint succeeds after repair");
    let child = create_entity_outcome(&runtime, "child-after-repair");
    assert_eq!(child.commit.parents, vec![commit_id]);

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(plan)
        .expect("fresh runtime recovers repaired parent and child");
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .expect("repaired lineage remains current")
            .commit_id,
        child.commit.commit_id
    );
    release_test_commit_snapshot(&runtime, &child);
}

/// Query releases the deferred commit's published snapshot before it repairs,
/// so repair must find the handle already gone and release nothing a second
/// time. The runtime's own obligation stays exactly once against that ordering.
#[test]
fn deferred_carrier_repair_is_a_no_op_after_the_snapshot_was_already_released() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "already-released-anchor");
    let published_before = runtime.visibility.published_snapshot_handle_count();
    let occupied_before = runtime
        .preparation_runtime_snapshot()
        .published_snapshot_count();

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("released-before-repair"),
            AllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(transaction, AllocationPolicy::SystemAllocation)
        .expect("already-released candidate prepares");
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("already-released candidate performs before its injected fault");
    };
    let commit_id = performed.canonical_commit().commit.commit_id;

    runtime.durability.arm_append_failure();
    let error = runtime
        .settle_performed_publication(performed)
        .expect_err("injected append fault defers settlement after performance");
    let settlement = error
        .deferred_settlement()
        .expect("the owner receives exact settlement repair authority");

    runtime
        .snapshots()
        .release_snapshot(&settlement.performed_result().snapshot)
        .expect("the result holder releases the published snapshot it was handed");
    assert_eq!(
        runtime.visibility.published_snapshot_handle_count(),
        published_before,
        "releasing through the result holder closes the published handle"
    );
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        occupied_before,
        "releasing through the result holder returns its capacity slot"
    );

    let repaired = runtime
        .repair_deferred_publication_settlement(settlement)
        .expect("repair completes the missing durable append after an early release");
    assert_eq!(repaired.commit_id, commit_id);
    assert_eq!(
        runtime.visibility.published_snapshot_handle_count(),
        published_before,
        "repair may not release a published handle that is already gone"
    );
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        occupied_before,
        "repair after an early release returns no second capacity slot"
    );
    assert!(
        runtime
            .snapshots()
            .release_snapshot(&settlement.performed_result().snapshot)
            .is_err(),
        "one published handle is released exactly once"
    );
    assert_eq!(
        runtime
            .durability()
            .durable_log()
            .iter()
            .filter(|entry| entry.envelope().commit.commit_id == commit_id)
            .count(),
        1,
        "an early release does not add or lose a durable append"
    );

    let child = create_entity_outcome(&runtime, "child-after-early-release");
    assert_eq!(child.commit.parents, vec![commit_id]);
    release_test_commit_snapshot(&runtime, &child);
}

mod external_capability_drop;
