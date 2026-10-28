use super::*;

fn fork(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, name: &str) {
    super::super::owner_service_phase3::fork_from_main(execution, runtime, name);
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
