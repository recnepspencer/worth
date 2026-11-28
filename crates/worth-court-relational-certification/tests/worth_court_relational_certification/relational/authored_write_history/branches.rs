use super::super::durable_publication::branch_history_publications::{
    create_entity_outcome_on_branch, fork, update_entity, update_entity_on_branch,
};
use super::{admit_main, collect, cursor, keys, limits, selected};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::{RelationalBranchBasisDenial, RelationalForkDenial},
    history::{BranchId, RelationalAuthoredWriteDenial},
    mvcc::{RelationalBranchTransactionAdmissionDenial, RelationalTransactionIntent},
};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    durable_publication::{create_entity_outcome, memory_runtime as runtime_with_test_schema},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn selected_prefix_excludes_siblings_and_foreign_sources() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = admit_main(&runtime).unwrap();
    fork(&execution, &runtime, "authored-sibling");
    let main = update_entity(&execution, &runtime, entity, "main-value");
    let main_basis = admit_main(&runtime).unwrap();
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
    for outcome in [&created, &main, &sibling] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn fixed_relevant_write_chain_excludes_growing_unrelated_and_sibling_history() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = admit_main(&runtime).unwrap();
    fork(&execution, &runtime, "cost-sibling");
    let one = update_entity(&execution, &runtime, entity, "one");
    let two = update_entity(&execution, &runtime, entity, "two");
    let target = admit_main(&runtime).unwrap();
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
    let target = admit_main(&runtime).unwrap();
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

#[test]
fn closed_original_and_target_keep_exact_history_until_cursor_release() {
    let host = serial_request();
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
    // Existing immutable custody can be extended; it is not fresh live admission.
    {
        let external = runtime.retain_component_basis(&original).unwrap();
        assert_eq!(
            runtime
                .readmit_retained_branch_basis(original.descriptor(), &external)
                .unwrap(),
            original
        );
        let bridge_hold = runtime.retain_observation_snapshot(&original).unwrap();
        assert_eq!(
            bridge_hold.observation().descriptor(),
            original.descriptor()
        );
        assert!(matches!(
            runtime.observe_branch(&original_identity),
            Err(RelationalBranchBasisDenial::UnknownBranch(_))
        ));
        assert!(matches!(
            runtime.readmit_branch_basis(original.descriptor()),
            Err(RelationalBranchBasisDenial::UnknownBranch(_))
        ));
        assert!(matches!(
            runtime.observe_fork_source(original_identity.branch_id()),
            Err(RelationalForkDenial::SourceBranchMissing)
        ));
        assert!(matches!(
            runtime.begin_branch_transaction(&original, RelationalTransactionIntent::ordinary()),
            Err(RelationalBranchTransactionAdmissionDenial::UnknownBranch)
        ));
    }
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
    drop(runtime.retain_component_basis(&target).unwrap());
    assert!(matches!(
        runtime.observe_branch(&target_identity),
        Err(RelationalBranchBasisDenial::UnknownBranch(_))
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
    let cursor = cursor(&runtime, execution, &original, &target, entity);
    drop(original);
    drop(target);
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
}
