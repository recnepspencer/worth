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
pub use super::carriage::ComputationPrior;
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationRead, SealedComputationFacts,
};
use crate::domain_computation::primary_graph::application_contribution::InstalledProducerEdition;
use crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge;

pub use worth_query_execution_contracts::facade::partitioned_tree_observation::{
    WorthQueryPartitionedComputationFullCause, WorthQueryPartitionedComputationRunForTest,
};

/// One installation of a partitioned computation's owner, compared by
/// identity. A reinstall with the same types and the same declaration may
/// run other owner or kernel code, so it is another installation and never
/// reuses this one's state. Clones of one installed computation are the same
/// installation. A basis holds the allocation it is compared by, so that
/// allocation outlives every state retained under it and no later
/// installation can share its address.
#[derive(Clone, Debug)]
pub struct ComputationInstallation(
    Arc<InstalledOwner>,
);

#[derive(Debug)]
struct InstalledOwner;

impl ComputationInstallation {
    pub fn new() -> Self {
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
pub struct RetainedBasis {
    installation: ComputationInstallation,
    edition: InstalledProducerEdition,
    input_digest: [u8; 32],
    value_bindings:
        worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
}

impl RetainedBasis {
    pub const fn new(
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
    pub fn with_input_digest_for_test(&self, input_digest: [u8; 32]) -> Self {
        Self {
            input_digest,
            ..self.clone()
        }
    }

    pub fn with_value_bindings_for_test(
        &self,
        value_bindings: worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    ) -> Self {
        Self {
            value_bindings,
            ..self.clone()
        }
    }

    /// Why `prior` is not this run's basis, or `None` when it is.
    pub fn drift(&self, prior: &Self) -> Option<WorthQueryPartitionedComputationFullCause> {
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
pub struct RetainedCall {
    pub charge: ComputationCallCharge,
    pub declared_units: Option<u64>,
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
pub struct RetainedPartition<Key> {
    pub key: Arc<Key>,
    pub gather: ComputationCallCharge,
}

/// The typed state: everything a later run of the same owner needs to skip
/// the membership, the unchanged items' keys and routes, and the unchanged
/// partitions.
pub struct RetainedPartitions<Key, Item, Reduced> {
    pub items: Items<Item>,
    pub digests: ItemDigests,
    /// The membership's call, and the declared work its items' digests spent.
    pub membership: RetainedCall,
    pub item_keys: Arc<worth_execution::ExecutionOrderedMap<PartitionItemId, RetainedCall>>,
    pub call_sequence: Arc<
        crate::domain_computation::primary_graph::invariant_projection::ComputationCallSequence,
    >,
    /// Every item's partition: the one routing a partition's members are read
    /// from.
    pub routing: Arc<ComputationPartitionRouting>,
    pub partitions: super::partition_metadata::PartitionMetadata<Key>,
    pub native_run: worth_execution::ExecutionRetainedRun<Reduced>,
    pub tree: NativeReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>,
}

pub use super::typed_prior::TypedPrior;

/// One run's state, retained on the record its attempt published.
pub struct RetainedComputation {
    pub basis: RetainedBasis,
    pub typed: Arc<dyn Any + Send + Sync>,
    pub facts: SealedComputationFacts,
    pub checked_calls:
        Option<crate::domain_computation::primary_graph::application_attempt::CheckedCallShape>,
}

impl RetainedComputation {
    pub fn original_facts(
        &self,
    ) -> crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload {
        self.facts.original_payload()
    }
}

/// A sealed run's state on its way to the record its attempt publishes.
pub struct SealedComputationRun {
    pub state: RetainedComputation,
    /// The custodied prior an incremental run built from. Publication consumes
    /// this holder before testing whether its exact displaced record is sole
    /// and unpinned; shared or pinned prior custody remains intact.
    pub cloned_from: Option<Arc<RetainedComputation>>,
    /// The request memory the state's tree and routing hold until the
    /// lineage charges the state.
    pub attribution_memory:
        Option<QueryMemoryReservation>,
}

/// The owner calls a run carried from `prior` without making them.
pub struct CarriedCalls {
    pub validated_at: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    pub prior: Arc<RetainedComputation>,
    pub membership: bool,
    pub items: CarriedItems,
    pub partitions: super::carried_partitions::CarriedPartitions,
    pub ranges: Option<
        Arc<crate::domain_computation::primary_graph::application_attempt::NativeCallCarriage>,
    >,
}

pub enum CarriedItems {
    Selected(BTreeSet<PartitionItemId>),
    All(Arc<worth_execution::ExecutionOrderedMap<PartitionItemId, RetainedCall>>),
}
impl CarriedItems {
    pub fn contains(&self, item: &PartitionItemId) -> bool {
        match self {
            Self::Selected(items) => items.contains(item),
            Self::All(items) => items.contains_key(item),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &PartitionItemId> {
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
    pub fn carried(&self, read: ComputationRead) -> bool {
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
pub struct CompletedComputationRun {
    pub basis: RetainedBasis,
    pub typed: Arc<dyn Any + Send + Sync>,
    /// The canonical shape and receipts of the actual owner calls.
    pub call_sequence: Arc<
        crate::domain_computation::primary_graph::invariant_projection::ComputationCallSequence,
    >,
    pub carried: Option<CarriedCalls>,
    /// Actual fact-index allocation, grown by sealing before index creation.
    pub attribution_memory: Option<QueryMemoryReservation>,
}

mod court_doors_for_test;
