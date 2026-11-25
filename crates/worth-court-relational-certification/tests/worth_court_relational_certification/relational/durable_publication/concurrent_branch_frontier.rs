//! Concurrent independent branch preparation from one Home frontier and the
//! preparation-vs-archive race (Store M4.1, D3a / S9).
//!
//! Branch A and Branch B both prepare against the same frontier and hold their
//! candidates before the FIFO patch-position reservation. Each publication
//! revalidates its own branch inside that reservation and composes only its
//! own branch delta onto the current frontier, so A's published root is never
//! overwritten by B's candidate.
use super::branch_history_publications::{fork, transaction};
use super::branch_history_recovery::{current_head, recover};
use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::*;
use worth_relational::facade::mvcc::{RelationalPublicationDenial, RelationalPublicationOutcome};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::*;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::{durable_runtime, field_patch};

fn branch(name: &str) -> BranchId {
    BranchId(name.into())
}

fn prepared(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    on: &str,
    key: &str,
) -> worth_relational::facade::mvcc::PreparedRelationalCommitCandidate {
    let mut txn = transaction(runtime, &branch(on));
    txn.push_batch(
        WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: key.into(),
                fields: field_patch("name", "name", key),
            },
        ))),
        execution.allocation_policy(),
    )
    .unwrap();
    runtime.prepare_branch_transaction(execution, txn).unwrap()
}

fn publish(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
) -> worth_relational::facade::history::CommitId {
    match runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    {
        RelationalPublicationOutcome::Performed(performed) => {
            let settled = runtime.settle_performed_publication(performed).unwrap();
            let commit = settled.commit.commit_id;
            worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot(
                runtime, &settled,
            );
            commit
        }
        other => panic!("independent branch publication must perform: {other:?}"),
    }
}

#[test]
fn branches_prepared_from_one_frontier_both_publish_and_both_survive_reopen() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    publish(
        execution,
        &runtime,
        prepared(execution, &runtime, "main", "seed"),
    );
    fork(execution, &runtime, "a");
    fork(execution, &runtime, "b");
    // Both candidates are prepared against frontier F0 before either cuts
    // over; B reserves its record slot first, A publishes first.
    let candidate_b = prepared(execution, &runtime, "b", "edit-b");
    let candidate_a = prepared(execution, &runtime, "a", "edit-a");
    let a = publish(execution, &runtime, candidate_a); // F0 -> F1
    let b = publish(execution, &runtime, candidate_b); // F1 -> F2, A retained
    assert_eq!(current_head(&runtime, &branch("a")).unwrap().commit_id, a);
    assert_eq!(current_head(&runtime, &branch("b")).unwrap().commit_id, b);
    let reopened = recover(execution, &medium, durable_runtime(&medium)).1;
    assert_eq!(current_head(&reopened, &branch("a")).unwrap().commit_id, a);
    assert_eq!(current_head(&reopened, &branch("b")).unwrap().commit_id, b);
}

#[test]
fn concurrent_publications_from_one_frontier_serialize_without_losing_either_edit() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    publish(
        execution,
        &runtime,
        prepared(execution, &runtime, "main", "seed"),
    );
    for name in ["a", "b"] {
        fork(execution, &runtime, name);
    }
    let candidates = [
        prepared(execution, &runtime, "a", "racing-a"),
        prepared(execution, &runtime, "b", "racing-b"),
    ];
    let commits = std::thread::scope(|scope| {
        let handles = candidates.map(|candidate| {
            let runtime = &runtime;
            scope.spawn(move || {
                let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
                let execution = ExecutionRequest::serial(&request);
                publish(&execution, runtime, candidate)
            })
        });
        handles.map(|handle| handle.join().expect("publisher thread"))
    });
    let reopened = recover(execution, &medium, durable_runtime(&medium)).1;
    for (name, commit) in ["a", "b"].into_iter().zip(commits) {
        assert_eq!(
            current_head(&reopened, &branch(name)).unwrap().commit_id,
            commit
        );
    }
}

#[test]
fn a_preparation_racing_archive_is_refused_with_archived_and_leaves_no_effect() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let mut runtime = durable_runtime(&medium);
    publish(
        execution,
        &runtime,
        prepared(execution, &runtime, "main", "seed"),
    );
    fork(execution, &runtime, "c");
    let head_of = |runtime: &RelationalRuntime| {
        runtime
            .branch_reference_state(&branch("c"))
            .unwrap()
            .observation()
            .target()
            .as_basis()
            .map(|head| (head.selected_commit_id(), head.roots().clone()))
    };
    let before = head_of(&runtime);
    let candidate = prepared(execution, &runtime, "c", "late-edit");
    let identity = runtime.branch_identity(&branch("c")).unwrap();
    runtime.archive_branch(execution, &identity).unwrap();
    assert!(matches!(
        runtime
            .publication_port()
            .compare_and_publish(execution, candidate),
        RelationalPublicationOutcome::Denied(RelationalPublicationDenial::Archived)
    ));
    let reopened = recover(execution, &medium, durable_runtime(&medium)).1;
    assert_eq!(
        reopened
            .branch_reference_state(&branch("c"))
            .unwrap()
            .lifecycle_posture(),
        worth_relational::facade::branch::RelationalBranchLifecyclePosture::Archived
    );
    assert_eq!(
        head_of(&reopened),
        before,
        "the refused candidate left the archived head untouched"
    );
}
