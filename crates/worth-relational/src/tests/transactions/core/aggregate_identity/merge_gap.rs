//! Independent Native branch reconciliation limitations preserve private ownership.
use super::fixtures::*;
use crate::facade::merge::{MergeExecutionPreparationError, MergeExecutionRequest, MergeIntent};
use crate::tests::support::*;

#[test]
fn native_branch_merge_readiness_blocks_unsupported_private_membership_changes() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime();
    let born = commit(execution, &runtime, "main", birth(&[1, 2, 3, 0]));
    let ids = members(&born, 4);
    create_branch_from_main(&runtime, "aggregate-source");
    let source = commit(
        execution,
        &runtime,
        "aggregate-source",
        rewire(
            execution,
            &runtime,
            &born.snapshot,
            &[(ids[1], ids[0]), (ids[3], ids[2])],
        ),
    );
    let before = runtime.history().branch_head(&BranchId("main".into()));
    let attempt = runtime.prepare_merge_execution(
        execution,
        MergeExecutionRequest {
            target_branch: BranchId("main".into()),
            source_branch: BranchId("aggregate-source".into()),
            merge_intent: MergeIntent::ReconcileIntoTarget,
        },
    );
    let Err(MergeExecutionPreparationError::NotExecutionReady(readiness)) = attempt else {
        panic!("unsupported Native aggregate branch reconciliation remains a typed refusal")
    };
    let target_read = runtime
        .read_truth()
        .read_snapshot(&born.snapshot, execution)
        .unwrap()
        .unwrap();
    let source_read = runtime
        .read_truth()
        .read_snapshot(&source.snapshot, execution)
        .unwrap()
        .unwrap();
    let mut blocked_private = 0;
    for denied in readiness.denied_records.iter() {
        let RecordRef::Relation(id) = denied.record else {
            continue;
        };
        let relation = target_read
            .get_relation(id)
            .or_else(|| source_read.get_relation(id))
            .unwrap();
        if relation.kind.kind_id == KindId(5) {
            blocked_private += 1;
        }
    }
    assert!(
        blocked_private > 0,
        "the actual readiness report includes protected membership records"
    );
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".into())),
        before
    );
    assert_eq!(
        records(execution, &runtime, &born.snapshot),
        records(
            execution,
            &runtime,
            &snapshot_for_owner_branch(&runtime, &BranchId("main".into()))
        )
    );
}
