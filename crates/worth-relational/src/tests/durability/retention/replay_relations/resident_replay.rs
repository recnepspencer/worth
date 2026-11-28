//! Asserts the retention lifecycle of a deleted relation with an explicit replay pin.

use super::*;

pub(super) fn assert_retention_lifecycle(execution: &RetentionRequest<'_, '_>) {
    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "replay-left");
    let target = create_entity(execution, &runtime, "replay-right");
    let created = create_relation_outcome(execution, &runtime, source, target, "replay-r1");
    let relation = changed_relations(&created)[0];
    let deleted = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("delete-relation").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(execution, &runtime).unwrap()
    };

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());
    assert!(runtime
        .history_authority()
        .retain_version_for_replay(execution, created.version_id)
        .unwrap());

    let pinned = runtime.retention().inspect_plan(execution).unwrap();
    assert_eq!(pinned.replay_pinned_relations, 1);
    assert_eq!(pinned.branch_pinned_relations, 0);
    assert_eq!(pinned.reclaimable_relations, 0);

    assert!(runtime
        .history_authority()
        .release_version_replay_retention(created.version_id));
    let released = runtime.retention().inspect_plan(execution).unwrap();
    assert_eq!(released.replay_pinned_relations, 0);
    assert_eq!(released.branch_pinned_relations, 0);
}
