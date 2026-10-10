use super::*;

#[test]
fn sole_branch_head_selects_commit_among_equivalent_retained_observations() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let committed = create_entity_outcome(&runtime.lock().unwrap(), "retained-head");
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .unwrap();
    let identity = runtime.lock().unwrap().main_branch_identity();
    let (_, basis) = source.observe_branch_basis(&identity).unwrap();
    let retained = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let head = source.bind_branch_head_basis_for_bridge(&basis).unwrap();

    let envelope = source
        .load_committed_patch(
            RelationalCommittedPatchRequest::new(TruthCommitIdentity::from_relational_commit_id(
                committed.commit.commit_id.0,
            )),
            execution,
        )
        .expect("the sole explicit head disambiguates equivalent observations");

    assert_eq!(envelope.snapshot_identity(), head.snapshot_identity());
    assert_ne!(envelope.snapshot_identity(), retained.snapshot_identity());
}

#[test]
fn barrier_ordered_rebind_cannot_be_removed_by_the_old_lease() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    create_entity_outcome(&runtime.lock().unwrap(), "race-first-head");
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .unwrap();
    let identity = runtime.lock().unwrap().main_branch_identity();
    let (_, old_basis) = source.observe_branch_basis(&identity).unwrap();
    let old_lease = source
        .bind_branch_head_basis_for_bridge(&old_basis)
        .unwrap();
    create_entity_outcome(&runtime.lock().unwrap(), "race-second-head");
    let (_, new_basis) = source.observe_branch_basis(&identity).unwrap();

    let start = Arc::new(Barrier::new(2));
    let rebound = Arc::new(Barrier::new(2));
    let bind_source = source.clone();
    let bind_start = Arc::clone(&start);
    let bind_rebound = Arc::clone(&rebound);
    let binder = std::thread::spawn(move || {
        bind_start.wait();
        let lease = bind_source
            .bind_branch_head_basis_for_bridge(&new_basis)
            .unwrap();
        bind_rebound.wait();
        lease
    });
    let release_start = Arc::clone(&start);
    let release_rebound = Arc::clone(&rebound);
    let releaser = std::thread::spawn(move || {
        release_start.wait();
        release_rebound.wait();
        old_lease.release()
    });

    let new_lease = binder.join().unwrap();
    let old_release = releaser.join().unwrap();
    assert!(!old_release.unbound());
    let current = source
        .load_branch_head_patch(
            &TruthBranchIdentity::from_relational_branch_id("main"),
            execution,
        )
        .unwrap();
    assert_eq!(current.snapshot_identity(), new_lease.snapshot_identity());
    assert!(new_lease.release().unbound());
}
