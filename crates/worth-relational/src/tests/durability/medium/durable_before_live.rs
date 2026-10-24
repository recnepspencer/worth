//! A commit reaches the medium before it reaches live state, and a refusal
//! from the medium leaves live state exactly as it was.

use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::medium::RelationalDurablePublicationRefusal;
use crate::facade::history::BranchId;
use crate::facade::transactions::TransactionCommitError;
use crate::mvcc::RelationalPublicationDenial;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;

/// Everything a commit could move, observed from outside the medium.
#[derive(Debug, PartialEq, Eq)]
struct LiveFootprint {
    main_head: Option<crate::history::data::RelationalCommitReceipt>,
    durable_log_len: usize,
    latest_patch: Option<PatchStreamPosition>,
    live_entities: usize,
    published_snapshot_handles: usize,
    published_snapshot_capacity: usize,
}

fn live_footprint(runtime: &RelationalRuntime) -> LiveFootprint {
    LiveFootprint {
        main_head: runtime.history().branch_head(&BranchId("main".to_owned())),
        durable_log_len: runtime.durability().durable_commit_count(),
        latest_patch: runtime
            .publication()
            .latest_patch()
            .map(|patch| patch.position),
        live_entities: runtime.storage_access().storage_stats().live_entities,
        published_snapshot_handles: runtime.visibility.published_snapshot_handle_count(),
        published_snapshot_capacity: runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
    }
}

fn commit_one(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> Result<crate::facade::transactions::CommitResult, TransactionCommitError> {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch_create(name),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction.commit(execution, runtime)
}

#[test]
fn a_refused_publication_was_offered_to_the_medium_and_moved_nothing_live() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = durable_runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "anchor");
    release_test_commit_snapshot(&runtime, &anchor);
    let medium = attached_test_medium(&runtime);
    let before = live_footprint(&runtime);

    medium.script_next_publication(ScriptedPublication::ProvenNoEffect);
    let refused = commit_one(execution, &runtime, "refused").expect_err("the medium refused");

    assert!(matches!(
        refused,
        TransactionCommitError::PublicationDenied {
            denial: RelationalPublicationDenial::DurablePublicationRefused(
                RelationalDurablePublicationRefusal::ProvenNoEffect { .. }
            ),
            ..
        }
    ));
    let offered = medium.offered().pop().expect("the medium was asked");
    assert_eq!(offered.position(), PatchStreamPosition(2));
    assert_eq!(live_footprint(&runtime), before);
    assert_eq!(medium.head(), PatchStreamPosition(1));
}

#[test]
fn a_refused_position_is_not_consumed() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = durable_runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "anchor");
    release_test_commit_snapshot(&runtime, &anchor);
    let medium = attached_test_medium(&runtime);

    medium.script_next_publication(ScriptedPublication::ProvenNoEffect);
    commit_one(execution, &runtime, "refused").expect_err("the medium refused");
    let refused = medium.offered().pop().expect("the medium was asked");
    let accepted = commit_one(execution, &runtime, "accepted").expect("the medium accepts");

    assert_eq!(accepted.patch_position(), PatchStreamPosition(2));
    assert_ne!(
        accepted.commit.commit_id,
        refused.commit_id(),
        "an identity a refused candidate held is burned, never reissued"
    );
    assert_eq!(accepted.commit.parents, vec![anchor.commit.commit_id]);
    assert_eq!(medium.head(), PatchStreamPosition(2));
    release_test_commit_snapshot(&runtime, &accepted);
}

#[test]
fn every_acknowledged_commit_is_the_artifact_the_medium_holds_at_its_position() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = durable_runtime_with_test_schema();
    let medium = attached_test_medium(&runtime);

    for name in ["first", "second", "third"] {
        let committed = create_entity_outcome(execution, &runtime, name);
        let held = medium
            .artifact_bytes(committed.patch_position())
            .expect("an acknowledged commit is already durable");
        let readmitted = RelationalCanonicalCommitArtifact::readmit(&held, execution)
            .expect("the medium holds Relational's own artifact");
        assert_eq!(readmitted.commit_id(), committed.commit.commit_id);
        assert_eq!(readmitted.position(), committed.patch_position());
        release_test_commit_snapshot(&runtime, &committed);
    }
    assert_eq!(
        medium.positions(),
        vec![
            PatchStreamPosition(1),
            PatchStreamPosition(2),
            PatchStreamPosition(3)
        ]
    );
}

#[test]
fn a_memory_only_runtime_publishes_without_a_medium() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_test_schema();

    let committed = create_entity_outcome(execution, &runtime, "memory-only");

    assert!(attached_durable_medium(&runtime).is_none());
    assert_eq!(runtime.durability().durable_commit_count(), 1);
    release_test_commit_snapshot(&runtime, &committed);
}
