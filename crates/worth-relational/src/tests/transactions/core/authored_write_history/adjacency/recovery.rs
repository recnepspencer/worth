//! The same canonical occurrences survive accelerator disposal and durable recovery.
use super::*;

#[test]
fn adjacency_occurrence_identities_rebuild_from_cold_canonical_history() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let a = create_entity(&execution, &runtime, "source");
    let b = create_entity(&execution, &runtime, "target");
    let c = create_entity(&execution, &runtime, "new-target");
    create_branch_from_main(&runtime, "adjacency-source");
    let source = test_owner_main_basis(&runtime).unwrap();
    let created = create_relation_outcome(&execution, &runtime, a, b, "edge");
    let relation = changed_relations(&created)[0];
    let moved = endpoints(&runtime, execution, relation, a, c);
    let noop = endpoints(&runtime, execution, relation, a, c);
    let removed =
        delete_relation_on_branch(&execution, &runtime, relation, BranchId("main".into()));
    let target = test_owner_main_basis(&runtime).unwrap();
    let expected = [
        created.commit.commit_id,
        moved.commit.commit_id,
        noop.commit.commit_id,
        removed.commit.commit_id,
    ];
    assert_eq!(
        scan(
            &runtime,
            execution,
            &source,
            &target,
            a,
            Direction::Outgoing
        ),
        expected
    );
    runtime
        .discard_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    runtime
        .rebuild_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    assert_eq!(
        scan(
            &runtime,
            execution,
            &source,
            &target,
            a,
            Direction::Outgoing
        ),
        expected
    );
    let (_, recovered) =
        checkpoint_and_recover_with(&execution, &runtime, durable_runtime_with_test_schema);
    let recovered_source = recovered
        .admit_branch_basis(
            &recovered
                .branch_identity(&BranchId("adjacency-source".into()))
                .unwrap(),
        )
        .unwrap();
    let recovered_target = test_owner_main_basis(&recovered).unwrap();
    recovered
        .rebuild_authored_write_generation(execution, selected(&recovered, &recovered_target))
        .unwrap();
    assert_eq!(
        scan(
            &recovered,
            execution,
            &recovered_source,
            &recovered_target,
            a,
            Direction::Outgoing
        ),
        expected
    );
    assert_eq!(
        scan(
            &recovered,
            execution,
            &recovered_source,
            &recovered_target,
            b,
            Direction::Incoming
        ),
        [created.commit.commit_id, moved.commit.commit_id]
    );
    assert_eq!(
        scan(
            &recovered,
            execution,
            &recovered_source,
            &recovered_target,
            c,
            Direction::Incoming
        ),
        [
            moved.commit.commit_id,
            noop.commit.commit_id,
            removed.commit.commit_id
        ]
    );
    for outcome in [&created, &moved, &noop, &removed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
