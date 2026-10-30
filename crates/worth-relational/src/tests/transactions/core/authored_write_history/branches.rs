use super::*;

fn fork(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, name: &str) {
    super::super::owner_service_phase3::fork_from_main(execution, runtime, name);
}

#[test]
fn closed_original_and_target_keep_exact_history_until_cursor_release() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let mut runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    fork(&execution, &runtime, "retained-original");
    fork(&execution, &runtime, "retained-target");
    let original_identity = runtime
        .branch_identity(&BranchId("retained-original".into()))
        .unwrap();
    let target_identity = runtime
        .branch_identity(&BranchId("retained-target".into()))
        .unwrap();
    let (_, original) = runtime.observe_branch(&original_identity).unwrap();
    let changed = update_entity_on_branch(
        &execution,
        &runtime,
        entity,
        "after",
        BranchId("retained-target".into()),
    );
    let (_, target) = runtime.observe_branch(&target_identity).unwrap();
    assert!(runtime
        .delete_branch(&execution, &original_identity)
        .unwrap()
        .deleted()
        .is_some());
    // The original's immutable ancestry remains usable while the descendant
    // is live. Closing a branch must not make fresh live admission possible.
    assert!(matches!(
        runtime.retain_component_basis(&original),
        Err(crate::branch::RelationalBranchBasisDenial::UnknownBranch(_))
    ));
    assert_eq!(
        collect(
            &runtime,
            execution,
            cursor(&runtime, execution, &original, &target, entity)
        )
        .0,
        [changed.commit.commit_id]
    );
    assert!(runtime
        .delete_branch(&execution, &target_identity)
        .unwrap()
        .deleted()
        .is_some());
    assert!(matches!(
        runtime.retain_component_basis(&target),
        Err(crate::branch::RelationalBranchBasisDenial::UnknownBranch(_))
    ));
    let older = runtime
        .select_reachable_commit(&target.observation(), created.commit.commit_id)
        .into_outcome()
        .unwrap();
    assert!(matches!(
        runtime.authored_write_history_cursor(execution, &original, older, &keys(entity)),
        Err(RelationalAuthoredWriteDenial::NotExactSelection)
    ));
    for outcome in [&created, &changed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
    let original_custody = std::sync::Arc::downgrade(&original.inner);
    let target_custody = std::sync::Arc::downgrade(&target.inner);
    let cursor = cursor(&runtime, execution, &original, &target, entity);
    drop(original);
    drop(target);
    assert!(original_custody.upgrade().is_some());
    assert!(target_custody.upgrade().is_some());
    let foreign = runtime_with_test_schema();
    assert!(matches!(
        cursor.prepare_next_page(&foreign, execution, limits(1)),
        Err(RelationalAuthoredWriteDenial::ForeignOwner)
    ));
    let cancelled = worth_execution::CancellationSource::new();
    let request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancelled.token(),
        None,
    );
    let page = cursor
        .prepare_next_page(&runtime, ExecutionRequest::serial(&request), limits(1))
        .unwrap();
    assert_eq!(
        page.page().entries().next().unwrap().commit_id(),
        changed.commit.commit_id
    );
    cancelled.cancel();
    assert!(matches!(
        cursor.accept_page(page),
        Err(RelationalAuthoredWriteDenial::Request(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        ))
    ));
    assert_eq!(
        collect(&runtime, execution, cursor).0,
        [changed.commit.commit_id]
    );
    assert!(
        original_custody.upgrade().is_none(),
        "the last cursor releases original custody"
    );
    assert!(
        target_custody.upgrade().is_none(),
        "the last cursor releases target custody"
    );
}

#[test]
fn selected_prefix_excludes_siblings_foreign_sources_and_unproven_merges() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = test_owner_main_basis(&runtime).unwrap();
    fork(&execution, &runtime, "authored-sibling");
    let main = update_entity(&execution, &runtime, entity, "main-value");
    let main_basis = test_owner_main_basis(&runtime).unwrap();
    let sibling = update_entity_on_branch(
        &execution,
        &runtime,
        entity,
        "sibling-value",
        BranchId("authored-sibling".into()),
    );
    let sibling_identity = runtime
        .branch_identity(&BranchId("authored-sibling".into()))
        .unwrap();
    let (_, sibling_basis) = runtime.observe_branch(&sibling_identity).unwrap();
    assert_eq!(
        collect(
            &runtime,
            execution,
            cursor(&runtime, execution, &original, &main_basis, entity)
        )
        .0,
        [main.commit.commit_id]
    );
    assert_eq!(
        collect(
            &runtime,
            execution,
            cursor(&runtime, execution, &original, &sibling_basis, entity)
        )
        .0,
        [sibling.commit.commit_id]
    );
    assert!(matches!(
        runtime.authored_write_history_cursor(
            execution,
            &main_basis,
            selected(&runtime, &sibling_basis),
            &keys(entity)
        ),
        Err(RelationalAuthoredWriteDenial::NotSelectedPrefix)
    ));
    let foreign = runtime_with_test_schema();
    assert!(matches!(
        foreign.authored_write_history_cursor(
            execution,
            &original,
            selected(&runtime, &main_basis),
            &keys(entity)
        ),
        Err(RelationalAuthoredWriteDenial::ForeignOwner)
    ));
    fork(&execution, &runtime, "authored-merge");
    let disjoint = create_entity_outcome_on_branch(
        &execution,
        &runtime,
        "merge-only-entity",
        BranchId("authored-merge".into()),
    );
    let transaction = test_owner_begin_merge_transaction(
        &runtime,
        BranchId("main".into()),
        [BranchId("authored-merge".into())],
    );
    let merged = transaction
        .commit(&execution, &runtime)
        .expect("genuine explicit-parent merge is admitted");
    let target = test_owner_main_basis(&runtime).unwrap();
    assert!(matches!(
        runtime.authored_write_history_cursor(
            execution,
            &original,
            selected(&runtime, &target),
            &keys(entity)
        ),
        Err(RelationalAuthoredWriteDenial::UnsupportedMergeInterval)
    ));
    for outcome in [&created, &main, &sibling, &disjoint, &merged] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn fixed_relevant_write_chain_excludes_growing_unrelated_and_sibling_history() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = test_owner_main_basis(&runtime).unwrap();
    fork(&execution, &runtime, "cost-sibling");
    let one = update_entity(&execution, &runtime, entity, "one");
    let two = update_entity(&execution, &runtime, entity, "two");
    let target = test_owner_main_basis(&runtime).unwrap();
    let before = collect(
        &runtime,
        execution,
        cursor(&runtime, execution, &original, &target, entity),
    );
    for index in 0..64 {
        let unrelated = create_entity_outcome(&execution, &runtime, &format!("unrelated-{index}"));
        release_test_commit_snapshot(&runtime, &unrelated);
        let sibling = update_entity_on_branch(
            &execution,
            &runtime,
            entity,
            &format!("sibling-{index}"),
            BranchId("cost-sibling".into()),
        );
        release_test_commit_snapshot(&runtime, &sibling);
    }
    let target = test_owner_main_basis(&runtime).unwrap();
    let after = collect(
        &runtime,
        execution,
        cursor(&runtime, execution, &original, &target, entity),
    );
    assert_eq!(
        after, before,
        "same actual relevant occurrences and source visits despite unrelated history"
    );
    assert_eq!(after.0, [one.commit.commit_id, two.commit.commit_id]);
    for outcome in [&created, &one, &two] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
