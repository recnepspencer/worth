//! What an owner declares about an input's partitions, and what one
//! partition's kernel is handed.

use worth_execution::{ChargedBytes, PartitionItemId};
use worth_foundational::facade::PartitionIdentity;

/// Which partition each input item belongs to.
pub struct WorthQueryComputationPartitionPlan<Key> {
    pub(super) shape: PlanShape<Key>,
}

/// The partitioner a plan names. Keyed grouping is the only one today.
pub(super) enum PlanShape<Key> {
    /// Each item's stable identity and partition key, in planned order.
    Keyed(Vec<(PartitionItemId, Key)>),
}

impl<Key> WorthQueryComputationPartitionPlan<Key> {
    /// Groups items by key: items whose keys have the same canonical encoding
    /// share a partition.
    ///
    /// `identity` names each item by a stable identity of the item itself,
    /// never its position, so a reordered input plans the same partitions with
    /// the same members in the same order. Naming two items alike is denied.
    pub fn keyed<Items>(
        items: Items,
        identity: impl Fn(&Items::Item) -> PartitionItemId,
        key: impl Fn(&Items::Item) -> Key,
    ) -> Self
    where
        Items: IntoIterator,
    {
        Self {
            shape: PlanShape::Keyed(
                items
                    .into_iter()
                    .map(|item| (identity(&item), key(&item)))
                    .collect(),
            ),
        }
    }
}

/// One item of a partition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryComputationPartitionItem {
    identity: PartitionItemId,
    position: usize,
}

impl WorthQueryComputationPartitionItem {
    pub(super) const fn new(identity: PartitionItemId, position: usize) -> Self {
        Self { identity, position }
    }

    /// The identity the plan gave the item.
    pub const fn identity(&self) -> PartitionItemId {
        self.identity
    }

    /// Where the plan met the item: its index in the items the plan was built
    /// from, for reading it back out of the input.
    pub const fn position(&self) -> usize {
        self.position
    }
}

/// One partition as its kernel sees it.
pub struct WorthQueryComputationPartitionView<'run, Key, Input> {
    members: &'run ComputationPartitionMembers<Key>,
    input: &'run Input,
}

impl<'run, Key, Input> WorthQueryComputationPartitionView<'run, Key, Input> {
    pub(super) const fn new(
        members: &'run ComputationPartitionMembers<Key>,
        input: &'run Input,
    ) -> Self {
        Self { members, input }
    }

    /// The partition's identity, derived from its key's canonical encoding.
    pub const fn identity(&self) -> PartitionIdentity {
        self.members.identity
    }

    /// The typed key of the partition: the key of its least item, whatever
    /// order the input holds the items in.
    pub const fn key(&self) -> &'run Key {
        &self.members.key
    }

    /// The whole input the partitions were planned over.
    pub const fn input(&self) -> &'run Input {
        self.input
    }

    /// The partition's items in ascending item identity order.
    pub fn items(&self) -> &'run [WorthQueryComputationPartitionItem] {
        &self.members.items
    }
}

/// One partition's gathered members: the value its kernel is dispatched with.
pub(super) struct ComputationPartitionMembers<Key> {
    pub(super) identity: PartitionIdentity,
    pub(super) key: Key,
    pub(super) items: Vec<WorthQueryComputationPartitionItem>,
}

/// The gathered item list is what a partition retains beyond its inline
/// value. A key's own heap storage is not visible here.
impl<Key> ChargedBytes for ComputationPartitionMembers<Key> {
    fn additional_charged_bytes(&self) -> u64 {
        self.items
            .capacity()
            .checked_mul(std::mem::size_of::<WorthQueryComputationPartitionItem>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .unwrap_or(u64::MAX)
    }
}
