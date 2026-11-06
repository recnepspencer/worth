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

use super::super::items::{ItemDigests, Items};
use super::super::routing::ComputationPartitionRouting;
pub(in crate::domain_computation) use super::carriage::ComputationPrior;
use {crate::domain_computation::primary_graph::vocabulary::application_attempt::computation_routing::ComputationRead, crate::domain_computation::primary_graph::application_attempt::SealedComputationFacts};
use crate::domain_computation::primary_graph::computation_retention::full_cause::WorthQueryPartitionedComputationFullCause;
use crate::domain_computation::primary_graph::computation_retention::retained_basis::RetainedBasis;
use crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge;
use crate::domain_computation::request_execution::QueryMemoryReservation;

/// How one run of a partitioned computation ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedComputationRunForTest {
    /// Only the partitions whose facts changed were gathered and computed.
    Incremental,
    /// Every partition was gathered and computed.
    Full(WorthQueryPartitionedComputationFullCause),
}

/// One owner call's charge on the reader and the declared work spent after
/// it on the call's behalf.
#[derive(Clone)]
pub struct RetainedCall {
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
pub struct RetainedPartition<Key> {
    pub(super) key: Arc<Key>,
    pub(super) gather: ComputationCallCharge,
}

/// The typed state: everything a later run of the same owner needs to skip
/// the membership, the unchanged items' keys and routes, and the unchanged
/// partitions.
pub struct RetainedPartitions<Key, Item, Reduced> {
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
pub struct RetainedComputation {
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
pub struct SealedComputationRun {
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

mod court_doors_for_test;

impl crate::domain_computation::request_execution::retained_dispatch::WorthQueryRetainedReductionObserver
    for WorthQueryPartitionedComputationFullCause
{
    fn observe_retained_reduction<R, F, E, Tree>(
        self,
        outcome: &worth_execution::RetainedReduceOutcome<R, F, E, Tree>,
    ) {
        super::observed::observe_retained_full_tree(self, outcome);
    }
}
