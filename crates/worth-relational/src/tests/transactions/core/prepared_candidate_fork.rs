use crate::tests::support::*;

#[test]
fn fork_resolves_a_port_performed_head_without_catalog_authority() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "fork-after-port-anchor");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("fork-after-port-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("port candidate prepares");
    let performed = match runtime.publication_port().compare_and_publish(candidate) {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("port candidate performs: {outcome:?}"),
    };
    let commit_id = performed.canonical_commit().commit.commit_id;
    let root_id = performed.next_basis().descriptor().root_identity();

    let (_, fork_source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("performed head issues a fork source");
    let forked = runtime
        .fork_branch(BranchId("after-port".to_owned()), fork_source)
        .expect("fork uses the exact root-owned canonical artifact");
    assert_eq!(forked.shared_commit_id(), Some(commit_id));
    let fork_basis = runtime
        .admit_branch_basis(forked.target_identity())
        .expect("forked basis is admitted");
    assert_eq!(fork_basis.descriptor().root_identity(), root_id);
    assert_eq!(fork_basis.observation().commit_id(), Some(commit_id));
    let committed = runtime
        .settle_performed_publication(performed)
        .expect("direct publication settles after branch-fork evidence");
    release_test_commit_snapshot(&runtime, &committed);
}

#[test]
fn runtime_fork_preserves_positioned_inventory_and_allocator_floor() {
    let runtime = runtime_with_test_schema();
    let source = create_entity_outcome(&runtime, "runtime-fork-source");
    let source_position = runtime
        .history
        .canonical_stream_position(source.commit.commit_id)
        .expect("source commit is positioned");

    let fork = runtime.fork().expect("settled runtime forks");
    assert_eq!(
        fork.history
            .canonical_stream_position(source.commit.commit_id),
        Some(source_position)
    );
    assert_eq!(
        fork.history()
            .historical_latest_commit()
            .map(|commit| commit.commit_id),
        Some(source.commit.commit_id)
    );

    let continued = create_entity_outcome(&fork, "runtime-fork-continuation");
    let continued_position = fork
        .history
        .canonical_stream_position(continued.commit.commit_id)
        .expect("fork continuation is positioned");
    assert!(source_position < continued_position);
    assert!(source.commit.commit_id < continued.commit.commit_id);
    assert!(source.commit.version_id < continued.commit.version_id);
    release_test_commit_snapshot(&runtime, &source);
    release_test_commit_snapshot(&fork, &continued);
}
