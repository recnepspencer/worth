//! Publication refusals: a conflicting write, a stale candidate and an
//! invalid transaction publish nothing; a commit the medium refuses before
//! any effect is denied and not installed; a commit whose durability is
//! unresolved blocks every later commit until the home is reopened, after
//! which its fate is resolved from the journal and commits resume.
//!
//! "Publishes nothing" is observed three ways: the live history, the
//! journal's `resolve` of the next position, and the history a reopened home
//! recovers.

use std::path::Path;

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::durability::RelationalCanonicalCommitArtifact;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::mvcc::{RelationalPublicationDenial, RelationalTransactionIntent};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::transactions::{
    RecordRef, TransactionCommitError, WorkerIntentBatch,
};
use worth_store::physical_runtime::certification::MediaOperationRole;
use worth_store_relational::RelationalStoreCommitFate;
use worth_store_relational::{RelationalStoreHome, RelationalStoreJournalRefusal};

use super::batches;
use super::durable_run::open;
use super::media_faults;
use super::schema_transition::SOURCE;
use super::workload;

fn main() -> BranchId {
    BranchId("main".to_owned())
}

fn create(
    home: &RelationalStoreHome,
    step: usize,
    execution: &ExecutionRequest<'_, '_>,
) -> EntityId {
    let outcome = workload::commit(home.runtime(), execution, &main(), batches::create(step))
        .expect("the create commits");
    match outcome.changed_records[..] {
        [RecordRef::Entity(entity)] => entity,
        ref other => panic!("a create changed {other:?}"),
    }
}

/// Require that nothing was published after `held` commits: not live, not in
/// the journal, and not after a reopen.
fn assert_published_nothing(
    home: RelationalStoreHome,
    location: &Path,
    held: usize,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) {
    let live = workload::artifacts_of(home.runtime());
    assert_eq!(live.len(), held, "{case}: live");
    if let Some(last) = live.last() {
        // Any commit's digest probes whether the next position is occupied.
        let probe = RelationalCanonicalCommitArtifact::from_untrusted_bytes(last.clone())
            .expect("an artifact frame")
            .payload_digest();
        assert_eq!(
            home.resolve(PatchStreamPosition(held as u64 + 1), probe)
                .unwrap(),
            RelationalStoreCommitFate::NotCommitted { head: held as u64 },
            "{case}: journal"
        );
    }
    home.close();
    let home = open(location, execution).expect("the home reopens");
    assert_eq!(
        workload::artifacts_of(home.runtime()).len(),
        held,
        "{case}: reopened"
    );
    home.close();
}

/// Two transactions staged on the same basis; the first commits.
fn race(
    home: &RelationalStoreHome,
    first: WorkerIntentBatch,
    second: WorkerIntentBatch,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), TransactionCommitError> {
    let runtime = home.runtime();
    let identity = runtime.branch_identity(&main()).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let mut transactions = [first, second].map(|batch| {
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .expect("a transaction on the shared basis");
        transaction
            .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
            .expect("the batch stages");
        Some(transaction)
    });
    transactions[0]
        .take()
        .unwrap()
        .commit(execution, runtime)
        .expect("the first commits");
    transactions[1]
        .take()
        .unwrap()
        .commit(execution, runtime)
        .map(|_| ())
}

#[test]
fn a_conflicting_write_and_a_stale_candidate_publish_nothing() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    for (case, disjoint) in [("conflicting write", false), ("stale candidate", true)] {
        let directory = tempfile::tempdir().unwrap();
        let location = directory.path().join("home");
        let home = open(&location, &execution).expect("a new home opens");
        let first = create(&home, 1, &execution);
        let second = create(&home, 2, &execution);
        let target = if disjoint { second } else { first };
        let raced = race(
            &home,
            batches::update(first, 10),
            batches::update(target, 11),
            &execution,
        );
        assert!(
            matches!(raced, Err(TransactionCommitError::Conflict { .. })),
            "{case}: {:?}",
            raced
        );
        assert_published_nothing(home, &location, 3, &execution, case);
    }
}

#[test]
fn an_invalid_transaction_publishes_nothing() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).expect("a new home opens");
    let live = create(&home, 1, &execution);
    let deleted = create(&home, 2, &execution);
    workload::commit(
        home.runtime(),
        &execution,
        &main(),
        batches::delete(deleted),
    )
    .expect("the delete commits");
    let invalid = workload::commit(
        home.runtime(),
        &execution,
        &main(),
        batches::relate(live, deleted, 3),
    );
    assert!(invalid.is_err(), "a relation to a deleted entity committed");
    assert_published_nothing(home, &location, 3, &execution, "invalid transaction");
}

#[test]
fn a_commit_the_medium_refuses_before_any_effect_is_denied_and_not_installed() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).expect("a new home opens");
    create(&home, 1, &execution);
    // A Store that has begun terminating admits no new publication.
    home.certification_serving()
        .certification_begin_lifecycle_termination();
    let denied = workload::commit(home.runtime(), &execution, &main(), batches::create(2));
    assert!(
        matches!(
            denied,
            Err(TransactionCommitError::PublicationDenied {
                denial: RelationalPublicationDenial::DurablePublicationRefused(_),
                ..
            })
        ),
        "{:?}",
        denied.map(|_| ())
    );
    assert_eq!(workload::artifacts_of(home.runtime()).len(), 1);
    home.certification_abort();
    let home = open(&location, &execution).expect("the home reopens");
    assert_eq!(
        workload::artifacts_of(home.runtime()).len(),
        1,
        "a denied commit recovered"
    );
    create(&home, 2, &execution);
    home.close();
}

#[test]
fn an_unresolved_commit_blocks_later_commits_until_reopen_then_resolves() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).expect("a new home opens");
    create(&home, 1, &execution);
    home.close();
    // The last positioned write of a commit happens after its WAL record is
    // durable: losing its answer leaves a durable commit of unknown fate.
    let (home, counters) =
        media_faults::open_with_faults(&location, SOURCE, &media_faults::counters(), &execution)
            .unwrap();
    media_faults::arm(&counters);
    create(&home, 2, &execution);
    let writes = media_faults::counts(&counters)
        .into_iter()
        .find(|(role, _)| *role == MediaOperationRole::PositionedWrite)
        .expect("a commit writes")
        .1;
    home.close();
    let fault = (
        MediaOperationRole::PositionedWrite,
        std::num::NonZeroU64::new(writes).unwrap(),
        worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
    );
    let (home, activations) =
        media_faults::open_with_faults(&location, SOURCE, &[fault], &execution).unwrap();
    media_faults::arm(&activations);
    let unresolved = workload::commit(home.runtime(), &execution, &main(), batches::create(3));
    let fate = unresolved
        .as_ref()
        .err()
        .and_then(TransactionCommitError::durability_unresolved_fate)
        .unwrap_or_else(|| {
            panic!(
                "the commit's fate is unresolved: {:?}",
                unresolved.as_ref().map(|_| ())
            )
        })
        .clone();
    assert_eq!(super::crash_oracle::commit_subject(&fate).0 .0, 3);
    assert_eq!(
        workload::artifacts_of(home.runtime()).len(),
        2,
        "installed while unresolved"
    );
    let blocked = workload::commit(home.runtime(), &execution, &main(), batches::create(4));
    assert_eq!(
        blocked
            .as_ref()
            .err()
            .and_then(TransactionCommitError::durability_unresolved_fate),
        Some(&fate),
        "a later commit was not blocked"
    );
    // In this incarnation the journal either knows the position is
    // unresolved or, once Store requires inspection, lends no journal at all;
    // it never answers committed.
    let in_incarnation = home.resolve(
        PatchStreamPosition(3),
        super::crash_oracle::commit_subject(&fate).2,
    );
    assert!(
        matches!(
            in_incarnation,
            Ok(RelationalStoreCommitFate::Unresolved)
                | Err(RelationalStoreJournalRefusal::Unavailable(_))
        ),
        "{in_incarnation:?}"
    );
    home.certification_abort();

    let home = open(&location, &execution).expect("the home reopens");
    let recovered = workload::artifacts_of(home.runtime());
    assert_eq!(
        recovered.len(),
        3,
        "the durable unresolved commit was recovered"
    );
    create(&home, 5, &execution);
    // The fate names the artifact by Relational's payload digest; resolving
    // it from the journal must find it committed.
    let resolved = home
        .resolve(
            PatchStreamPosition(3),
            super::crash_oracle::commit_subject(&fate).2,
        )
        .unwrap();
    assert!(
        matches!(resolved, RelationalStoreCommitFate::Committed),
        "the unresolved fate resolved {resolved:?} after reopen"
    );
    home.close();
}
