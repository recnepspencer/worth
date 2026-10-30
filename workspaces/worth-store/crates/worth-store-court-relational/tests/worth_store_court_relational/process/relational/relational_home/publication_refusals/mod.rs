//! Publication refusals: a conflicting write, a stale candidate and an
//! invalid transaction publish nothing; a commit the medium refuses before
//! any effect is denied and not installed; a commit whose durability is
//! unresolved blocks every later commit until the home is reopened, after
//! which its fate is resolved from the journal and commits resume.
//!
//! "Publishes nothing" is observed three ways: the live history, the
//! journal's `resolve` of the next position, and the history a reopened home
//! recovers.

#[allow(unused_imports)]
use crate::publication_refusals::*;
use std::path::Path;

use worth_relational::facade::durability::RelationalCanonicalCommitArtifact;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::publication::PatchStreamPosition;
use worth_store::physical_runtime::certification::MediaOperationRole;
use worth_store_relational::RelationalStoreCommitFate;
use {worth_execution::ExecutionAllocationPolicy, worth_execution::ExecutionRequest};
use {
    worth_relational::facade::mvcc::RelationalPublicationDenial,
    worth_relational::facade::mvcc::RelationalTransactionIntent,
};
use {
    worth_relational::facade::transactions::RecordRef,
    worth_relational::facade::transactions::TransactionCommitError,
    worth_relational::facade::transactions::WorkerIntentBatch,
};
use {
    worth_store_relational::RelationalStoreHome,
    worth_store_relational::RelationalStoreJournalRefusal,
};

use crate::batches;
use crate::durable_run::open;
use crate::media_faults;
use crate::schema_transition::SOURCE;
use crate::workload;

#[test]
fn a_conflicting_write_and_a_stale_candidate_publish_nothing() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    for (case, disjoint) in [("conflicting write", false), ("stale candidate", true)] {
        let directory = tempfile::tempdir().unwrap();
        let location = directory.path().join("home");
        let home = open(&location, &execution).expect("a new home opens");
        let first = create(&home, 1, &execution);
        let second = create(&home, 2, &execution);
        let target = if disjoint { second } else { first };
        let raced = race(
            &home,
            crate::batches::update(first, 10),
            crate::batches::update(target, 11),
            &execution,
        );
        assert!(
            matches!(
                raced,
                Err(
                    worth_relational::facade::transactions::TransactionCommitError::Conflict { .. }
                )
            ),
            "{case}: {:?}",
            raced
        );
        assert_published_nothing(home, &location, 3, &execution, case);
    }
}

#[test]
fn an_invalid_transaction_publishes_nothing() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).expect("a new home opens");
    let live = create(&home, 1, &execution);
    let deleted = create(&home, 2, &execution);
    crate::workload::commit(
        home.runtime(),
        &execution,
        &main(),
        crate::batches::delete(deleted),
    )
    .expect("the delete commits");
    let invalid = crate::workload::commit(
        home.runtime(),
        &execution,
        &main(),
        crate::batches::relate(live, deleted, 3),
    );
    assert!(invalid.is_err(), "a relation to a deleted entity committed");
    assert_published_nothing(home, &location, 3, &execution, "invalid transaction");
}

#[test]
fn a_commit_the_medium_refuses_before_any_effect_is_denied_and_not_installed() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).expect("a new home opens");
    create(&home, 1, &execution);
    // A Store that has begun terminating admits no new publication.
    home.certification_serving()
        .certification_begin_lifecycle_termination();
    let denied = crate::workload::commit(
        home.runtime(),
        &execution,
        &main(),
        crate::batches::create(2),
    );
    assert!(
        matches!(
            denied,
            Err(worth_relational::facade::transactions::TransactionCommitError::PublicationDenied {
                denial: worth_relational::facade::mvcc::RelationalPublicationDenial::DurablePublicationRefused(_),
                ..
            })
        ),
        "{:?}",
        denied.map(|_| ())
    );
    assert_eq!(crate::workload::artifacts_of(home.runtime()).len(), 1);
    home.certification_abort();
    let home = open(&location, &execution).expect("the home reopens");
    assert_eq!(
        crate::workload::artifacts_of(home.runtime()).len(),
        1,
        "a denied commit recovered"
    );
    create(&home, 2, &execution);
    home.close();
}

#[test]
fn an_unresolved_commit_blocks_later_commits_until_reopen_then_resolves() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).expect("a new home opens");
    create(&home, 1, &execution);
    home.close();
    // The last positioned write of a commit happens after its WAL record is
    // durable: losing its answer leaves a durable commit of unknown fate.
    let (home, counters) = crate::media_faults::open_with_faults(
        &location,
        SOURCE,
        &crate::media_faults::counters(),
        &execution,
    )
    .unwrap();
    crate::media_faults::arm(&counters);
    create(&home, 2, &execution);
    let writes = crate::media_faults::counts(&counters)
        .into_iter()
        .find(|(role, _)| {
            *role
                == worth_store::physical_runtime::certification::MediaOperationRole::PositionedWrite
        })
        .expect("a commit writes")
        .1;
    home.close();
    let fault = (
        worth_store::physical_runtime::certification::MediaOperationRole::PositionedWrite,
        std::num::NonZeroU64::new(writes).unwrap(),
        worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
    );
    let (home, activations) =
        crate::media_faults::open_with_faults(&location, SOURCE, &[fault], &execution).unwrap();
    let acknowledged = crate::workload::artifacts_of(home.runtime());
    crate::media_faults::arm(&activations);
    let unresolved = crate::workload::commit(
        home.runtime(),
        &execution,
        &main(),
        crate::batches::create(3),
    );
    let fate = unresolved
        .as_ref()
        .err()
        .and_then(worth_relational::facade::transactions::TransactionCommitError::durability_unresolved_fate)
        .unwrap_or_else(|| {
            panic!(
                "the commit's fate is unresolved: {:?}",
                unresolved.as_ref().map(|_| ())
            )
        })
        .clone();
    assert_eq!(crate::crash_oracle::commit_subject(&fate).0 .0, 3);
    assert_eq!(
        crate::workload::append_live_artifacts(home.runtime(), &acknowledged).len(),
        2,
        "installed while unresolved"
    );
    let blocked = crate::workload::commit(
        home.runtime(),
        &execution,
        &main(),
        crate::batches::create(4),
    );
    assert_eq!(
        blocked
            .as_ref()
            .err()
            .and_then(worth_relational::facade::transactions::TransactionCommitError::durability_unresolved_fate),
        Some(&fate),
        "a later commit was not blocked"
    );
    // In this incarnation the journal either knows the position is
    // unresolved or, once Store requires inspection, lends no journal at all;
    // it never answers committed.
    let in_incarnation = home.resolve(
        PatchStreamPosition(3),
        crate::crash_oracle::commit_subject(&fate).2,
    );
    assert!(
        matches!(
            in_incarnation,
            Ok(worth_store_relational::RelationalStoreCommitFate::Unresolved)
                | Err(worth_store_relational::RelationalStoreJournalRefusal::Unavailable(_))
        ),
        "{in_incarnation:?}"
    );
    home.certification_abort();

    let home = open(&location, &execution).expect("the home reopens");
    let recovered = crate::workload::artifacts_of(home.runtime());
    assert_eq!(
        recovered.len(),
        3,
        "the durable unresolved commit was recovered"
    );
    assert!(
        recovered.starts_with(&acknowledged),
        "recovery preserves the acknowledged artifact bytes"
    );
    create(&home, 5, &execution);
    // The fate names the artifact by Relational's payload digest; resolving
    // it from the journal must find it committed.
    let resolved = home
        .resolve(
            PatchStreamPosition(3),
            crate::crash_oracle::commit_subject(&fate).2,
        )
        .unwrap();
    assert!(
        matches!(
            resolved,
            worth_store_relational::RelationalStoreCommitFate::Committed
        ),
        "the unresolved fate resolved {resolved:?} after reopen"
    );
    home.close();
}
