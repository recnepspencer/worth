//! What a partitioned computation keeps between the runs of the producer that
//! runs it, and why a run recomputed everything.
//!
//! A run that completed under a producer leaves its items and their digests,
//! its routing, its partitions' keys, the work and reach of every owner call,
//! each kernel's work, its reduction tree and the facts the calls read as
//! seal observed them. The next run receives the selected state's custodied
//! handle. The comparator alone decides whether its calls can be carried.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_execution::{ChargedBytes, PartitionItemId, ReductionTree};
use worth_foundational::facade::PartitionIdentity;

use super::super::super::request_execution::QueryMemoryReservation;
use super::super::items::{ItemDigests, Items};
use super::super::routing::ComputationPartitionRouting;
pub(in crate::domain_computation) use super::carriage::ComputationPrior;
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationRead, SealedComputationFacts,
};
use crate::domain_computation::primary_graph::application_contribution::InstalledProducerEdition;
use crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge;
use crate::domain_computation::primary_graph::output_lineage::CustodiedComputation;

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
    /// The observation bound exceeds the run's declared work.
    ObservationOverBudget,
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
        Some(if self.installation != prior.installation {
            WorthQueryPartitionedComputationFullCause::OtherInstallation
        } else if self.edition != prior.edition {
            WorthQueryPartitionedComputationFullCause::OtherEdition
        } else {
            WorthQueryPartitionedComputationFullCause::InputChanged
        })
    }
}

/// One owner call's charge on the reader and the declared work spent after
/// it on the call's behalf.
#[derive(Clone)]
pub(super) struct RetainedCall {
    pub(super) charge: ComputationCallCharge,
    pub(super) declared_units: u64,
}

/// One partition of a run.
pub(super) struct RetainedPartition<Key> {
    pub(super) key: Arc<Key>,
    /// The key's canonical encoding length: what retaining it is charged.
    pub(super) key_bytes: u64,
    pub(super) gather: ComputationCallCharge,
    pub(super) kernel_units: u64,
}

/// The typed state: everything a later run of the same owner needs to skip
/// the membership, the unchanged items' keys and routes, and the unchanged
/// partitions.
pub(super) struct RetainedPartitions<Key, Item, Reduced> {
    pub(super) items: Items<Item>,
    pub(super) digests: ItemDigests,
    /// The membership's call, and the declared work its items' digests spent.
    pub(super) membership: RetainedCall,
    pub(super) item_keys: Arc<BTreeMap<PartitionItemId, RetainedCall>>,
    /// Every item's partition: the one routing a partition's members are read
    /// from.
    pub(super) routing: Arc<ComputationPartitionRouting>,
    pub(super) partitions: BTreeMap<PartitionIdentity, Arc<RetainedPartition<Key>>>,
    pub(super) tree: ReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>,
}

impl<Key, Item, Reduced> RetainedPartitions<Key, Item, Reduced>
where
    Item: ChargedBytes,
    Reduced: ChargedBytes,
{
    /// What the state holds: every item and its digest, every call's charge
    /// and reach, the routing as the partitioner charges it, every
    /// partition's key at its canonical encoding length, and the reduction
    /// tree as execution charges it. `None` when the sum overflows.
    pub(super) fn charged_bytes(&self) -> Option<u64> {
        let size = |bytes: usize| u64::try_from(bytes).ok();
        let digest = size(std::mem::size_of::<(PartitionItemId, [u8; 32])>())?;
        let items = self.items.values().try_fold(0_u64, |sum, item| {
            sum.checked_add(size(std::mem::size_of::<(PartitionItemId, Item)>())?)?
                .checked_add(item.additional_charged_bytes())?
                .checked_add(digest)
        })?;
        let item_keys = self.item_keys.values().try_fold(items, |sum, call| {
            sum.checked_add(size(std::mem::size_of::<(PartitionItemId, RetainedCall)>())?)?
                .checked_add(call.charge.additional_bytes()?)
        })?;
        let partitions = self
            .partitions
            .values()
            .try_fold(item_keys, |sum, partition| {
                sum.checked_add(size(std::mem::size_of::<RetainedPartition<Key>>())?)?
                    .checked_add(partition.key_bytes)?
                    .checked_add(partition.gather.additional_bytes()?)
            })?;
        partitions
            .checked_add(self.membership.charge.additional_bytes()?)?
            .checked_add(self.routing.charged_bytes()?)?
            .checked_add(self.tree.additional_charged_bytes())
    }
}

/// A prior run's state, its typed half downcast once, where the comparator
/// took it.
pub(super) struct TypedPrior<Key, Item, Reduced> {
    pub(super) typed: Arc<RetainedPartitions<Key, Item, Reduced>>,
    // Field drop order releases the independent typed Arc before its custody.
    pub(super) state: Arc<CustodiedComputation>,
}

/// One run's state, retained on the record its attempt published.
pub(in crate::domain_computation) struct RetainedComputation {
    pub(super) basis: RetainedBasis,
    pub(super) typed: Arc<dyn Any + Send + Sync>,
    pub(super) facts: SealedComputationFacts,
    /// What the state is charged on the lineage ledger, or `None` when the
    /// sum has no value.
    pub(super) bytes: Option<u64>,
}

impl RetainedComputation {
    /// The completed state's payload charge. Custody adds its own Arc
    /// allocation, and every holder shares that full ticket until final release.
    /// `None` when the payload sum overflows: the incoming state is evicted.
    pub(in crate::domain_computation::primary_graph) const fn retained_bytes(&self) -> Option<u64> {
        self.bytes
    }
}

/// A sealed run's state on its way to the record its attempt publishes.
pub(in crate::domain_computation) struct SealedComputationRun {
    pub(in crate::domain_computation::primary_graph) state: RetainedComputation,
    /// The custodied prior an incremental run built from. Publication consumes
    /// this holder before testing whether its exact displaced record is sole
    /// and unpinned; shared or pinned prior custody remains intact.
    pub(in crate::domain_computation::primary_graph) cloned_from: Option<Arc<CustodiedComputation>>,
    /// The request memory the state's tree and routing hold until the
    /// lineage charges the state.
    pub(in crate::domain_computation::primary_graph) tree_memory: RunTreeMemory,
}

/// Request memory a run's tree and its new routing hold from their build
/// until the record that keeps them is charged.
pub(in crate::domain_computation) struct RunTreeMemory {
    pub(super) _held: QueryMemoryReservation,
    pub(super) _routing: Option<QueryMemoryReservation>,
}

/// The owner calls a run carried from `prior` without making them.
pub(super) struct CarriedCalls {
    pub(super) prior: Arc<CustodiedComputation>,
    pub(super) membership: bool,
    pub(super) items: BTreeSet<PartitionItemId>,
    pub(super) partitions: BTreeSet<PartitionIdentity>,
}

impl CarriedCalls {
    /// Whether the run carried `read`.
    pub(super) fn carried(&self, read: ComputationRead) -> bool {
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
    /// The typed state's bytes, `None` when the sum overflows.
    pub(super) typed_bytes: Option<u64>,
    pub(super) carried: Option<CarriedCalls>,
    /// The request memory the run's tree holds.
    pub(super) tree_memory: QueryMemoryReservation,
    /// The request memory a routing the run built holds, `None` when it
    /// built none.
    pub(super) routing_memory: Option<QueryMemoryReservation>,
}

#[cfg(test)]
impl RetainedComputation {
    pub(in crate::domain_computation::primary_graph) fn overflow_bytes_for_test(&mut self) {
        self.bytes = None;
    }

    pub(in crate::domain_computation::primary_graph) fn fixture_byte_formula(&self) -> u64 {
        use super::super::attribution_tests::{Number, Parity};
        use worth_execution::KeyedPartitioner;
        let typed = self
            .typed
            .downcast_ref::<RetainedPartitions<Parity, Number, u64>>()
            .unwrap();
        assert_eq!(
            (
                typed.items.len(),
                typed.item_keys.len(),
                typed.partitions.len()
            ),
            (4, 4, 2)
        );
        let size = |bytes: usize| u64::try_from(bytes).unwrap();
        let items = 4
            * (size(std::mem::size_of::<(PartitionItemId, Number)>())
                + size(std::mem::size_of::<(PartitionItemId, [u8; 32])>()));
        let keys = 4 * size(std::mem::size_of::<(PartitionItemId, RetainedCall)>())
            + typed
                .item_keys
                .values()
                .map(|call| call.charge.additional_bytes().unwrap())
                .sum::<u64>();
        // Parity(0/1): newtype tag, one-byte name length, name, unsigned
        // tag, and a one-byte varint value. Domain framing is not key storage.
        let parity_bytes = 1 + 1 + size("Parity".len()) + 1 + 1;
        let partitions = 2
            * (size(std::mem::size_of::<RetainedPartition<Parity>>()) + parity_bytes)
            + typed
                .partitions
                .values()
                .map(|part| part.gather.additional_bytes().unwrap())
                .sum::<u64>();
        // Each immutable u64 node contains its identity, value, aggregate,
        // two Arc child links and retained-byte sum, plus two Arc counters.
        let node = size(std::mem::size_of::<PartitionIdentity>())
            + 3 * size(std::mem::size_of::<u64>())
            + 4 * size(std::mem::size_of::<usize>());
        // The state's one Arc allocation contains its inline state and ticket,
        // with the two Arc counters and alignment padding declared by Arc.
        use crate::domain_computation::primary_graph::output_lineage::CustodiedComputation;
        let alignment =
            std::mem::align_of::<CustodiedComputation>().max(std::mem::align_of::<usize>());
        let header = 2 * std::mem::size_of::<usize>();
        let offset = header.div_ceil(alignment) * alignment;
        let capsule =
            (offset + std::mem::size_of::<CustodiedComputation>()).div_ceil(alignment) * alignment;
        items
            + keys
            + partitions
            + typed.membership.charge.additional_bytes().unwrap()
            + KeyedPartitioner::<[u8; 32]>::retained_bytes(4, 2, 0).unwrap()
            + 2 * node
            + self.facts.charged_bytes().unwrap()
            + size(capsule)
    }
}
