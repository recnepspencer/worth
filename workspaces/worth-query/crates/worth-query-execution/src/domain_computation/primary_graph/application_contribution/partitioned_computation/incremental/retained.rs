//! What a partitioned computation keeps between the runs of the producer that
//! runs it, and why a run recomputed everything.
//!
//! A run that completed under a producer leaves its items and their digests,
//! its routing, its partitions' keys, the work and reach of every owner call,
//! each kernel's work, its reduction tree and the facts the calls read as
//! seal observed them. The next run receives the selected state's custodied
//! handle. The comparator alone decides whether its calls can be carried.

use std::any::Any;
use std::collections::BTreeSet;
use std::sync::Arc;

use worth_execution::{NativeReductionTree, PartitionItemId};

use super::super::super::request_execution::QueryMemoryReservation;
use super::super::items::{ItemDigests, Items};
use super::super::routing::ComputationPartitionRouting;
pub(in crate::domain_computation) use super::carriage::ComputationPrior;
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationRead, SealedComputationFacts,
};
use crate::domain_computation::primary_graph::application_contribution::InstalledProducerEdition;
use crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge;

/// How one run of a partitioned computation ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedComputationRunForTest {
    /// Only the partitions whose facts changed were gathered and computed.
    Incremental,
    /// Every partition was gathered and computed.
    Full(WorthQueryPartitionedComputationFullCause),
}

/// Why a run recomputed every partition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedComputationFullCause {
    /// No output record has been published at this address.
    FirstRun,
    /// Checkpoint restoration retained no computation state.
    Restored,
    /// Republication retained facts but no computation state.
    Republished,
    /// The decision ran no partitioned computation.
    NotProduced,
    /// The run's charge or retained byte sum could not be measured.
    Unmeasured,
    /// A successor took this record's computation reservation.
    Moved,
    /// The lineage ledger refused the computation reservation.
    Evicted,
    /// The input value's canonical encoding differs.
    InputChanged,
    /// The installed owner differs, even when its types match.
    OtherInstallation,
    /// The installed producer edition differs.
    OtherEdition,
    /// The observation bound exceeds the request's work policy.
    ObservationOverBudget,
    /// Complete structural reads require fresh issuance under this admission;
    /// their live carried-value handoff is not yet qualified for reuse.
    StructuralWalkRead,
    /// Aggregate decision inputs require fresh Native identity and membership reads.
    AggregateRead,
    /// A prior was already consumed by an earlier invocation in this handler.
    NoPriorHanded,
    /// This reader belongs to an operation outside a producer invocation.
    NoProducerPrior,
    /// Policy suppressed the completed computation's retention.
    RetentionPolicy,
    /// The handler ran several computations and has no single attribution.
    SeveralComputations,
    /// A collision restart computed without a reusable retention basis.
    CollisionSuppressed,
    /// The computation stopped before leaving completed state.
    Stopped,
    /// Execution policy chose not to retain this computation.
    Unretained,
    /// Incremental routing met a compact partition identity collision.
    IdentityCollision,
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
#[derive(Clone, Debug)]
pub(super) struct RetainedBasis {
    installation: ComputationInstallation,
    edition: InstalledProducerEdition,
    input_digest: [u8; 32],
    value_bindings:
        worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
}

impl RetainedBasis {
    pub(super) const fn new(
        installation: ComputationInstallation,
        edition: InstalledProducerEdition,
        input_digest: [u8; 32],
        value_bindings: worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    ) -> Self {
        Self {
            installation,
            edition,
            input_digest,
            value_bindings,
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

    #[cfg(test)]
    pub(super) fn with_value_bindings_for_test(
        &self,
        value_bindings: worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    ) -> Self {
        Self {
            value_bindings,
            ..self.clone()
        }
    }

    /// Why `prior` is not this run's basis, or `None` when it is.
    pub(super) fn drift(&self, prior: &Self) -> Option<WorthQueryPartitionedComputationFullCause> {
        if self.installation != prior.installation
            || !self
                .value_bindings
                .same_installation_as(&prior.value_bindings)
        {
            Some(WorthQueryPartitionedComputationFullCause::OtherInstallation)
        } else if self.edition != prior.edition {
            Some(WorthQueryPartitionedComputationFullCause::OtherEdition)
        } else if self.input_digest != prior.input_digest {
            Some(WorthQueryPartitionedComputationFullCause::InputChanged)
        } else {
            None
        }
    }
}

/// One owner call's charge on the reader and the declared work spent after
/// it on the call's behalf.
#[derive(Clone)]
pub(super) struct RetainedCall {
    pub(super) charge: ComputationCallCharge,
    pub(super) declared_units: Option<u64>,
}

impl worth_execution::ChargedBytes for RetainedCall {
    fn additional_charged_bytes(&self) -> u64 {
        self.charge.additional_bytes().unwrap_or(u64::MAX)
    }
}
impl<Key> Clone for RetainedPartition<Key> {
    fn clone(&self) -> Self {
        Self {
            key: Arc::clone(&self.key),
            gather: self.gather.clone(),
        }
    }
}
/// One partition of a run.
pub(super) struct RetainedPartition<Key> {
    pub(super) key: Arc<Key>,
    pub(super) gather: ComputationCallCharge,
}

/// The typed state: everything a later run of the same owner needs to skip
/// the membership, the unchanged items' keys and routes, and the unchanged
/// partitions.
pub(super) struct RetainedPartitions<Key, Item, Reduced> {
    pub(super) items: Items<Item>,
    pub(super) digests: ItemDigests,
    /// The membership's call, and the declared work its items' digests spent.
    pub(super) membership: RetainedCall,
    pub(super) item_keys: Arc<worth_execution::ExecutionOrderedMap<PartitionItemId, RetainedCall>>,
    pub(super) call_sequence: Arc<
        crate::domain_computation::primary_graph::invariant_projection::ComputationCallSequence,
    >,
    /// Every item's partition: the one routing a partition's members are read
    /// from.
    pub(super) routing: Arc<ComputationPartitionRouting>,
    pub(super) partitions: super::partition_metadata::PartitionMetadata<Key>,
    pub(super) native_run: worth_execution::ExecutionRetainedRun<Reduced>,
    pub(super) tree: NativeReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>,
}

pub(super) use super::typed_prior::TypedPrior;

/// One run's state, retained on the record its attempt published.
pub(in crate::domain_computation) struct RetainedComputation {
    pub(super) basis: RetainedBasis,
    pub(super) typed: Arc<dyn Any + Send + Sync>,
    pub(super) facts: SealedComputationFacts,
    pub(super) checked_calls:
        Option<crate::domain_computation::primary_graph::application_attempt::CheckedCallShape>,
}

impl RetainedComputation {
    pub(in crate::domain_computation::primary_graph) fn original_facts(
        &self,
    ) -> crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload {
        self.facts.original_payload()
    }
}

/// A sealed run's state on its way to the record its attempt publishes.
pub(in crate::domain_computation) struct SealedComputationRun {
    pub(in crate::domain_computation::primary_graph) state: RetainedComputation,
    /// The custodied prior an incremental run built from. Publication consumes
    /// this holder before testing whether its exact displaced record is sole
    /// and unpinned; shared or pinned prior custody remains intact.
    pub(in crate::domain_computation::primary_graph) cloned_from: Option<Arc<RetainedComputation>>,
    /// The request memory the state's tree and routing hold until the
    /// lineage charges the state.
    pub(in crate::domain_computation::primary_graph) attribution_memory:
        Option<QueryMemoryReservation>,
}

/// The owner calls a run carried from `prior` without making them.
pub(super) struct CarriedCalls {
    pub(super) validated_at: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    pub(super) prior: Arc<RetainedComputation>,
    pub(super) membership: bool,
    pub(super) items: CarriedItems,
    pub(super) partitions: super::carried_partitions::CarriedPartitions,
    pub(super) ranges: Option<
        Arc<crate::domain_computation::primary_graph::application_attempt::NativeCallCarriage>,
    >,
}

pub(super) enum CarriedItems {
    Selected(BTreeSet<PartitionItemId>),
    All(Arc<worth_execution::ExecutionOrderedMap<PartitionItemId, RetainedCall>>),
}
impl CarriedItems {
    pub(super) fn contains(&self, item: &PartitionItemId) -> bool {
        match self {
            Self::Selected(items) => items.contains(item),
            Self::All(items) => items.contains_key(item),
        }
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = &PartitionItemId> {
        let (selected, all) = match self {
            Self::Selected(items) => (Some(items), None),
            Self::All(items) => (None, Some(items)),
        };
        selected
            .into_iter()
            .flat_map(|items| items.iter())
            .chain(all.into_iter().flat_map(|items| items.keys()))
    }
}

impl CarriedCalls {
    /// Whether the run carried `read`.
    pub(super) fn carried(&self, read: ComputationRead) -> bool {
        if let Some(ranges) = &self.ranges {
            return ranges.carried(read);
        }
        match read {
            ComputationRead::Membership => self.membership,
            ComputationRead::ItemKey(item) => self.items.contains(&item),
            ComputationRead::Partition(partition) => self.partitions.contains(&partition),
        }
    }
}

/// What a completed run leaves for seal.
pub(in crate::domain_computation::primary_graph) struct CompletedComputationRun {
    pub(super) basis: RetainedBasis,
    pub(super) typed: Arc<dyn Any + Send + Sync>,
    /// The canonical shape and receipts of the actual owner calls.
    pub(super) call_sequence: Arc<
        crate::domain_computation::primary_graph::invariant_projection::ComputationCallSequence,
    >,
    pub(super) carried: Option<CarriedCalls>,
    /// Actual fact-index allocation, grown by sealing before index creation.
    pub(super) attribution_memory: Option<QueryMemoryReservation>,
}
