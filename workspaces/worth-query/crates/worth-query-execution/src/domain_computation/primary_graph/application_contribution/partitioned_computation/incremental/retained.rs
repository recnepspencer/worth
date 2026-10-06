//! What a partitioned computation keeps between the runs of the producer that
//! runs it, and why a run recomputed everything.
//!
//! A run that completed under a producer leaves its items, its partitions'
//! keys and members, the work and reach of every owner call, each kernel's
//! work, its reduction tree and the facts the calls read as seal observed
//! them. The next run of the same producer is handed that state by value from
//! the record it selected. Nothing outside the comparator module reads it.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use worth_execution::{ChargedBytes, PartitionItemId, ReductionTree};
use worth_foundational::facade::{ExecutionReport, PartitionIdentity};

use crate::domain_computation::primary_graph::application_attempt::SealedComputationFacts;
use crate::domain_computation::primary_graph::application_contribution::InstalledProducerEdition;
use crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge;
use crate::domain_computation::primary_graph::output_lineage::PriorComputationRecord;

/// How one run of a partitioned computation ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedComputationRun {
    /// Only the partitions whose facts changed were gathered and computed.
    Incremental,
    /// Every partition was gathered and computed.
    Full(WorthQueryPartitionedComputationFullCause),
}

/// Why a run recomputed every partition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedComputationFullCause {
    /// The selected record retained no state of this computation: a first
    /// run, a restored or aliased record, another computation, owner or
    /// producer edition, or a run that is no producer's.
    NoPriorRecord,
    /// The input value's canonical encoding differs.
    InputChanged,
    /// A fact the membership or an item's key read changed.
    PartitionerRebuilt,
    /// The last run's state did not fit what may be retained, or held more
    /// facts than this run's declared work may compare.
    Evicted,
}

/// One installation of a partitioned computation's owner, compared by
/// identity. A reinstall with the same types and the same declaration may
/// run other owner or kernel code, so it is another installation and never
/// reuses this one's state. Clones of one installed computation are the same
/// installation. A basis holds the allocation it is compared by, so that
/// allocation outlives every state retained under it and no later
/// installation can share its address.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct ComputationInstallation(
    Arc<InstalledOwner>,
);

#[derive(Debug)]
struct InstalledOwner;

impl ComputationInstallation {
    pub(in crate::domain_computation::primary_graph) fn new() -> Self {
        Self(Arc::new(InstalledOwner))
    }
}

impl PartialEq for ComputationInstallation {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ComputationInstallation {}

/// What a retained run is the same run for: the owner's installation, the
/// producer edition and the input value's digest. The installation fixes
/// the feature, computation and owner types, so a state is never another
/// owner's, nor another installation's of the same owner type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RetainedBasis {
    installation: ComputationInstallation,
    edition: InstalledProducerEdition,
    input_digest: [u8; 32],
}

impl RetainedBasis {
    pub(super) const fn new(
        installation: ComputationInstallation,
        edition: InstalledProducerEdition,
        input_digest: [u8; 32],
    ) -> Self {
        Self {
            installation,
            edition,
            input_digest,
        }
    }

    /// This basis over another input value.
    #[cfg(test)]
    pub(super) fn with_input_digest_for_test(&self, input_digest: [u8; 32]) -> Self {
        Self {
            input_digest,
            ..self.clone()
        }
    }

    /// Why `prior` is not this run's basis, or `None` when it is.
    pub(super) fn drift(&self, prior: &Self) -> Option<WorthQueryPartitionedComputationFullCause> {
        if self == prior {
            return None;
        }
        let only_input = self.installation == prior.installation && self.edition == prior.edition;
        Some(if only_input {
            WorthQueryPartitionedComputationFullCause::InputChanged
        } else {
            WorthQueryPartitionedComputationFullCause::NoPriorRecord
        })
    }
}

/// One owner call's charge on the reader and the declared work spent after
/// it on the call's behalf.
pub(super) struct RetainedCall {
    pub(super) charge: ComputationCallCharge,
    pub(super) declared_units: u64,
}

/// One partition of a run.
pub(super) struct RetainedPartition<Key> {
    pub(super) key: Arc<Key>,
    /// The key's canonical encoding length: what retaining it is charged.
    pub(super) key_bytes: u64,
    pub(super) members: Arc<[PartitionItemId]>,
    pub(super) gather: ComputationCallCharge,
    pub(super) kernel_units: u64,
}

/// The typed state: everything a later run of the same owner needs to skip
/// the membership, the keys and the unchanged partitions.
pub(super) struct RetainedPartitions<Key, Item, Reduced> {
    pub(super) items: Arc<BTreeMap<PartitionItemId, Item>>,
    pub(super) membership: ComputationCallCharge,
    pub(super) item_keys: Arc<BTreeMap<PartitionItemId, RetainedCall>>,
    pub(super) partitions: BTreeMap<PartitionIdentity, Arc<RetainedPartition<Key>>>,
    pub(super) tree: ReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>,
    /// The combine work a full build of the tree charges.
    pub(super) reduction_work: u64,
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced>
where
    Item: ChargedBytes,
    Reduced: ChargedBytes,
{
    /// What the state holds: every item, every call's charge and reach,
    /// every partition's key at its canonical encoding length and its
    /// members, and the reduction tree as execution charges it. `None` when
    /// the sum overflows.
    pub(super) fn charged_bytes(&self) -> Option<u64> {
        let size = |bytes: usize| u64::try_from(bytes).ok();
        let items = self.items.values().try_fold(0_u64, |sum, item| {
            sum.checked_add(size(std::mem::size_of::<(PartitionItemId, Item)>())?)?
                .checked_add(item.additional_charged_bytes())
        })?;
        let item_keys = self.item_keys.values().try_fold(items, |sum, call| {
            sum.checked_add(size(std::mem::size_of::<(PartitionItemId, RetainedCall)>())?)?
                .checked_add(call.charge.additional_bytes()?)
        })?;
        let partitions = self
            .partitions
            .values()
            .try_fold(item_keys, |sum, partition| {
                let members = size(partition.members.len())?
                    .checked_mul(size(std::mem::size_of::<PartitionItemId>())?)?;
                sum.checked_add(size(std::mem::size_of::<RetainedPartition<Key>>())?)?
                    .checked_add(partition.key_bytes)?
                    .checked_add(members)?
                    .checked_add(partition.gather.additional_bytes()?)
            })?;
        partitions
            .checked_add(self.membership.additional_bytes()?)?
            .checked_add(self.tree.additional_charged_bytes())
    }
}

/// A prior run's state, its typed half downcast once, where the comparator
/// took it.
pub(super) struct TypedPrior<Key, Item, Reduced> {
    pub(super) state: Arc<RetainedComputation>,
    pub(super) typed: Arc<RetainedPartitions<Key, Item, Reduced>>,
}

/// One run's state, retained on the record its attempt published.
pub(in crate::domain_computation) struct RetainedComputation {
    pub(super) basis: RetainedBasis,
    pub(super) typed: Arc<dyn Any + Send + Sync>,
    pub(super) facts: SealedComputationFacts,
    /// What the state is charged on the lineage ledger, or `None` when the
    /// sum has no value.
    bytes: Option<u64>,
}

impl RetainedComputation {
    /// The bytes the lineage ledger reserves for the state while a record
    /// holds it. `None` when the sum overflows: the state is evicted.
    pub(in crate::domain_computation::primary_graph) const fn retained_bytes(&self) -> Option<u64> {
        self.bytes
    }
}

/// A sealed run's state on its way to the record its attempt publishes.
pub(in crate::domain_computation) struct SealedComputationRun {
    pub(in crate::domain_computation::primary_graph) state: RetainedComputation,
    /// The prior state an incremental run built its tree from. Publication
    /// moves the ledger reservation only from the record still holding
    /// exactly this state.
    pub(in crate::domain_computation::primary_graph) cloned_from: Option<Arc<RetainedComputation>>,
}

/// The partitions a run carried from `prior` without gathering them.
pub(super) struct CarriedPartitions {
    pub(super) prior: Arc<RetainedComputation>,
    pub(super) skipped: BTreeSet<PartitionIdentity>,
}

/// What a completed run leaves for seal.
pub(in crate::domain_computation::primary_graph) struct CompletedComputationRun {
    pub(super) basis: RetainedBasis,
    /// `None` when the run measured something it cannot carry again.
    pub(super) typed: Option<Arc<dyn Any + Send + Sync>>,
    /// The typed state's bytes, `None` when the sum overflows.
    pub(super) typed_bytes: Option<u64>,
    pub(super) carried: Option<CarriedPartitions>,
}

/// Where a completed run leaves itself for seal.
pub(in crate::domain_computation::primary_graph) type ComputationDeposit =
    Arc<Mutex<Option<CompletedComputationRun>>>;

/// What a producer hands the partitioned computation its handler runs: the
/// installed edition, the state the selected record retained, and that
/// record.
#[derive(Clone)]
pub(in crate::domain_computation) struct ComputationPrior {
    pub(super) edition: InstalledProducerEdition,
    pub(super) retained:
        Result<Arc<RetainedComputation>, WorthQueryPartitionedComputationFullCause>,
    record: Option<PriorComputationRecord>,
}

impl ComputationPrior {
    pub(in crate::domain_computation::primary_graph) const fn new(
        edition: InstalledProducerEdition,
        retained: Result<Arc<RetainedComputation>, WorthQueryPartitionedComputationFullCause>,
        record: Option<PriorComputationRecord>,
    ) -> Self {
        Self {
            edition,
            retained,
            record,
        }
    }

    /// The record the state was selected from.
    pub(in crate::domain_computation::primary_graph) fn into_record(
        self,
    ) -> Option<PriorComputationRecord> {
        self.record
    }
}

impl CompletedComputationRun {
    /// Seals the run over the facts seal observed. Every fact a carried call
    /// read must be the fact the prior run read: the law that makes a skipped
    /// partition's result the result a full run computes. `Err` when it is
    /// not, and `Ok(None)` when the run has nothing to retain.
    pub(in crate::domain_computation::primary_graph) fn seal(
        self,
        facts: SealedComputationFacts,
    ) -> Result<Option<SealedComputationRun>, ()> {
        if let Some(carried) = &self.carried {
            let unchanged = carried
                .prior
                .facts
                .facts()
                .filter(|(_, _, readers)| readers.read_by_carried(&carried.skipped))
                .all(|(key, fact, _)| facts.fact(key) == Some(fact));
            if !unchanged {
                return Err(());
            }
        }
        let bytes = self
            .typed_bytes
            .zip(facts.charged_bytes())
            .and_then(|(typed, facts)| typed.checked_add(facts));
        Ok(self.typed.map(|typed| SealedComputationRun {
            state: RetainedComputation {
                basis: self.basis,
                typed,
                facts,
                bytes,
            },
            cloned_from: self.carried.map(|carried| carried.prior),
        }))
    }
}

#[cfg(test)]
thread_local! {
    static PRIOR_IN_TEST: std::cell::RefCell<Option<ComputationPrior>> =
        const { std::cell::RefCell::new(None) };
    static SEALED_IN_TEST: std::cell::RefCell<Option<SealedComputationRun>> =
        const { std::cell::RefCell::new(None) };
}

/// An attempt no producer runs has no prior and keeps no run. A test hands
/// it both here.
#[cfg(test)]
impl ComputationPrior {
    pub(in crate::domain_computation) fn handed_in_test() -> Option<Self> {
        PRIOR_IN_TEST.with(|prior| prior.borrow().clone())
    }

    pub(super) fn hand_in_test(prior: Option<Self>) {
        PRIOR_IN_TEST.with(|handed| *handed.borrow_mut() = prior);
    }
}

#[cfg(test)]
impl SealedComputationRun {
    pub(in crate::domain_computation) fn keep_in_test(run: Option<Self>) {
        SEALED_IN_TEST.with(|kept| *kept.borrow_mut() = run);
    }

    pub(super) fn kept_in_test() -> Option<Self> {
        SEALED_IN_TEST.with(|kept| kept.borrow_mut().take())
    }
}

#[cfg(any(test, feature = "test-query-execution-observer"))]
thread_local! {
    static RUNS: std::cell::RefCell<Vec<(WorthQueryPartitionedComputationRun, Option<ExecutionReport>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Shows a completed run to the test observer. The handler and the owner
/// never see how a run ran.
pub(super) fn observe(run: WorthQueryPartitionedComputationRun, report: Option<ExecutionReport>) {
    #[cfg(any(test, feature = "test-query-execution-observer"))]
    RUNS.with(|runs| runs.borrow_mut().push((run, report)));
    #[cfg(not(any(test, feature = "test-query-execution-observer")))]
    let _ = (run, report);
}

/// Takes every partitioned computation run completed on this thread since the
/// last call: how it ran, and execution's report of a full run.
#[cfg(any(test, feature = "test-query-execution-observer"))]
pub fn partitioned_computation_runs_on_this_thread_for_test(
) -> Vec<(WorthQueryPartitionedComputationRun, Option<ExecutionReport>)> {
    RUNS.with(|runs| std::mem::take(&mut *runs.borrow_mut()))
}
