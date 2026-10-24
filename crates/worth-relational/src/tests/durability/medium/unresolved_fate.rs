//! A publication whose durable fate the medium cannot tell installs nothing,
//! blocks the runtime, and is resolved by reopening from the medium.

use crate::facade::history::BranchId;
use crate::facade::transactions::{CommitResult, TransactionCommitError};
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;

fn commit_one(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> Result<CommitResult, TransactionCommitError> {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch_create(name),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction.commit(execution, runtime)
}

fn main_head(runtime: &RelationalRuntime) -> Option<crate::history::data::RelationalCommitReceipt> {
    runtime.history().branch_head(&BranchId("main".to_owned()))
}

#[test]
fn an_unresolved_publication_installs_nothing_and_refuses_every_later_commit_alike() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = durable_runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "anchor");
    release_test_commit_snapshot(&runtime, &anchor);
    let medium = attached_test_medium(&runtime);
    let published_handles = runtime.visibility.published_snapshot_handle_count();

    medium.script_next_publication(ScriptedPublication::IndeterminateWithoutEffect);
    let unresolved = commit_one(execution, &runtime, "unresolved").expect_err("fate unknown");
    let fate = unresolved
        .durability_unresolved_fate()
        .expect("the error names the unresolved publication")
        .clone();
    let offered = medium.offered().pop().expect("the medium was asked");
    assert_eq!(fate.position(), PatchStreamPosition(2));
    assert_eq!(fate.position(), offered.position());
    assert_eq!(fate.commit_id(), offered.commit_id());
    assert_eq!(fate.digest(), offered.payload_digest());
    assert_eq!(main_head(&runtime), Some(anchor.commit.clone()));
    assert_eq!(runtime.durability().durable_commit_count(), 1);
    assert_eq!(
        runtime.visibility.published_snapshot_handle_count(),
        published_handles
    );

    let offered_before_retry = medium.offered().len();
    let later = commit_one(execution, &runtime, "later").expect_err("still unresolved");
    assert!(matches!(
        later,
        TransactionCommitError::DurabilityUnresolved { .. }
    ));
    assert_eq!(later.durability_unresolved_fate(), Some(&fate));
    assert_eq!(
        medium.offered().len(),
        offered_before_retry,
        "a later position is never offered over an unresolved one"
    );
    assert_eq!(main_head(&runtime), Some(anchor.commit.clone()));
}

#[test]
fn reopening_resolves_an_absent_unresolved_commit_as_never_committed() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = durable_runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "anchor");
    release_test_commit_snapshot(&runtime, &anchor);
    let medium = attached_test_medium(&runtime);
    medium.script_next_publication(ScriptedPublication::IndeterminateWithoutEffect);
    let fate = commit_one(execution, &runtime, "lost")
        .expect_err("fate unknown")
        .durability_unresolved_fate()
        .expect("unresolved")
        .clone();
    assert!(medium.artifact_bytes(fate.position()).is_none());

    let (recovery, reopened) = recover_with(execution, &runtime, durable_runtime_with_test_schema);

    assert_eq!(recovery.recovered_commits, 1);
    assert_eq!(main_head(&reopened), Some(anchor.commit.clone()));
    let resumed = commit_one(execution, &reopened, "resumed").expect("a reopened home commits");
    assert_eq!(resumed.patch_position(), fate.position());
    assert_eq!(resumed.commit.parents, vec![anchor.commit.commit_id]);
    assert_eq!(medium.head(), fate.position());
    release_test_commit_snapshot(&reopened, &resumed);
}

#[test]
fn reopening_resolves_a_present_unresolved_commit_as_committed_exactly_once() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = durable_runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "anchor");
    release_test_commit_snapshot(&runtime, &anchor);
    let medium = attached_test_medium(&runtime);
    medium.script_next_publication(ScriptedPublication::IndeterminateAfterEffect);
    let fate = commit_one(execution, &runtime, "durable-but-unknown")
        .expect_err("fate unknown")
        .durability_unresolved_fate()
        .expect("unresolved")
        .clone();
    assert_eq!(medium.head(), fate.position());
    assert_eq!(main_head(&runtime), Some(anchor.commit.clone()));

    let (recovery, reopened) = recover_with(execution, &runtime, durable_runtime_with_test_schema);

    assert_eq!(recovery.recovered_commits, 2);
    let head = main_head(&reopened).expect("the durable commit is current");
    assert_eq!(head.commit_id, fate.commit_id());
    assert_eq!(
        reopened
            .durability()
            .durable_log()
            .iter()
            .filter(|entry| entry.envelope().commit.commit_id == fate.commit_id())
            .count(),
        1
    );
    let next = commit_one(execution, &reopened, "after-resolution").expect("commits resume");
    assert_eq!(
        next.patch_position(),
        PatchStreamPosition(fate.position().0 + 1)
    );
    assert_eq!(next.commit.parents, vec![fate.commit_id()]);
    assert_ne!(next.commit.commit_id, fate.commit_id());
    release_test_commit_snapshot(&reopened, &next);
}
