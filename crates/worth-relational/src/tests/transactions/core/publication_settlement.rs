use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::tests::support::*;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

#[test]
fn direct_publication_settles_before_a_child_can_acknowledge() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "settlement-anchor");
    crate::tests::support::checkpoint_durably(execution, &runtime)
        .expect("checkpoint seals the baseline before the direct parent");

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("direct-parent"),
            AllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .expect("direct parent prepares");
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
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
            .read_patch_stream(execution, PatchStreamRequest::default())
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
        .prepare_branch_transaction(execution, blocked_child)
        .expect_err("a child cannot hide its parent's unfinished settlement");
    assert!(format!("{child_denial:?}").contains("requires explicit owner settlement"));
    let settled_parent = runtime
        .settle_performed_publication(performed)
        .expect("the direct owner explicitly settles its performed publication");
    release_test_commit_snapshot(&runtime, &settled_parent);

    let child = create_entity_outcome(execution, &runtime, "ordinary-child");
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

    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    bind_durable_medium(&mut recovered, attached_test_medium(&runtime).attachment());
    recovered
        .durability_recovery()
        .recover(execution, plan)
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
    let continued = create_entity_outcome(execution, &recovered, "post-recovery-child");
    let continued_position = recovered
        .history
        .canonical_stream_position(continued.commit.commit_id)
        .expect("post-recovery child receives a stream position");
    assert!(child_position < continued_position);
    release_test_commit_snapshot(&recovered, &continued);
}
