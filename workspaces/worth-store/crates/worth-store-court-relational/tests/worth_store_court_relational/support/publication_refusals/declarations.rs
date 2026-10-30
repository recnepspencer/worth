//! Publication refusals: a conflicting write, a stale candidate and an
//! invalid transaction publish nothing; a commit the medium refuses before
//! any effect is denied and not installed; a commit whose durability is
//! unresolved blocks every later commit until the home is reopened, after
//! which its fate is resolved from the journal and commits resume.
//!
//! "Publishes nothing" is observed three ways: the live history, the
//! journal's `resolve` of the next position, and the history a reopened home
//! recovers.

pub(crate) use std::path::Path;

pub(crate) use worth_relational::facade::durability::RelationalCanonicalCommitArtifact;
pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use worth_relational::facade::identity::EntityId;
pub(crate) use worth_relational::facade::publication::PatchStreamPosition;
pub(crate) use worth_store::physical_runtime::certification::MediaOperationRole;
pub(crate) use worth_store_relational::RelationalStoreCommitFate;
pub(crate) use {worth_execution::ExecutionAllocationPolicy, worth_execution::ExecutionRequest};
pub(crate) use {
    worth_relational::facade::mvcc::RelationalPublicationDenial,
    worth_relational::facade::mvcc::RelationalTransactionIntent,
};
pub(crate) use {
    worth_relational::facade::transactions::RecordRef,
    worth_relational::facade::transactions::TransactionCommitError,
    worth_relational::facade::transactions::WorkerIntentBatch,
};
pub(crate) use {
    worth_store_relational::RelationalStoreHome,
    worth_store_relational::RelationalStoreJournalRefusal,
};

pub(crate) use crate::batches;
pub(crate) use crate::durable_run::open;
pub(crate) use crate::media_faults;
pub(crate) use crate::schema_transition::SOURCE;
pub(crate) use crate::workload;

pub(crate) fn main() -> BranchId {
    BranchId("main".to_owned())
}

pub(crate) fn create(
    home: &RelationalStoreHome,
    step: usize,
    execution: &ExecutionRequest<'_, '_>,
) -> EntityId {
    let outcome = crate::workload::commit(
        home.runtime(),
        execution,
        &main(),
        crate::batches::create(step),
    )
    .expect("the create commits");
    match outcome.changed_records[..] {
        [worth_relational::facade::transactions::RecordRef::Entity(entity)] => entity,
        ref other => panic!("a create changed {other:?}"),
    }
}

/// Require that nothing was published after `held` commits: not live, not in
/// the journal, and not after a reopen.
pub(crate) fn assert_published_nothing(
    home: RelationalStoreHome,
    location: &Path,
    held: usize,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) {
    let live = crate::workload::artifacts_of(home.runtime());
    assert_eq!(live.len(), held, "{case}: live");
    if let Some(last) = live.last() {
        // Any commit's digest probes whether the next position is occupied.
        let probe = worth_relational::facade::durability::RelationalCanonicalCommitArtifact::from_untrusted_bytes(last.clone())
            .expect("an artifact frame")
            .payload_digest();
        assert_eq!(
            home.resolve(PatchStreamPosition(held as u64 + 1), probe)
                .unwrap(),
            worth_store_relational::RelationalStoreCommitFate::NotCommitted { head: held as u64 },
            "{case}: journal"
        );
    }
    home.close();
    let home = open(location, execution).expect("the home reopens");
    assert_eq!(
        crate::workload::artifacts_of(home.runtime()).len(),
        held,
        "{case}: reopened"
    );
    home.close();
}

/// Two transactions staged on the same basis; the first commits.
pub(crate) fn race(
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
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("a transaction on the shared basis");
        transaction
            .push_batch(
                batch,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
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
