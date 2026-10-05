//! What an owner declares about an input's items, and what one partition's
//! gathering and kernel are handed.

use std::collections::BTreeMap;

use worth_execution::{ChargedBytes, PartitionItemId};
use worth_foundational::facade::PartitionIdentity;

/// The items an input holds.
pub struct WorthQueryComputationPartitionPlan<Item> {
    pub(super) shape: PlanShape<Item>,
}

/// The partitioner a plan names. Keyed grouping is the only one today.
pub(super) enum PlanShape<Item> {
    /// Each item with its stable identity, in planned order. The owner's
    /// `partition_key` keys each one.
    Keyed(Vec<(PartitionItemId, Item)>),
}

impl<Item> WorthQueryComputationPartitionPlan<Item> {
    /// Groups items by key: items whose keys have the same canonical encoding
    /// share a partition.
    ///
    /// `identity` names each item by a stable identity of the item itself,
    /// never its position, so a reordered input plans the same partitions with
    /// the same members in the same order. Naming two items alike is denied.
    pub fn keyed<Items>(items: Items, identity: impl Fn(&Item) -> PartitionItemId) -> Self
    where
        Items: IntoIterator<Item = Item>,
    {
        Self {
            shape: PlanShape::Keyed(
                items
                    .into_iter()
                    .map(|item| (identity(&item), item))
                    .collect(),
            ),
        }
    }
}

/// One partition as its gathering sees it: its key and the items in it.
pub struct WorthQueryComputationPartitionMembers<'gather, Key, Item> {
    identity: PartitionIdentity,
    key: &'gather Key,
    members: &'gather [PartitionItemId],
    items: &'gather BTreeMap<PartitionItemId, Item>,
}

impl<'gather, Key, Item> WorthQueryComputationPartitionMembers<'gather, Key, Item> {
    pub(super) const fn new(
        identity: PartitionIdentity,
        key: &'gather Key,
        members: &'gather [PartitionItemId],
        items: &'gather BTreeMap<PartitionItemId, Item>,
    ) -> Self {
        Self {
            identity,
            key,
            members,
            items,
        }
    }

    /// The partition's identity, derived from its key's canonical encoding.
    pub const fn identity(&self) -> PartitionIdentity {
        self.identity
    }

    /// The typed key of the partition: the key of its least item, whatever
    /// order the input holds the items in.
    pub const fn key(&self) -> &'gather Key {
        self.key
    }

    /// The partition's items in ascending item identity order.
    pub fn items(&self) -> impl Iterator<Item = (PartitionItemId, &'gather Item)> + '_ {
        self.members
            .iter()
            .map(|member| (*member, &self.items[member]))
    }
}

/// One partition as its kernel sees it.
pub struct WorthQueryComputationPartitionView<'run, Key, Gathered> {
    partition: &'run GatheredComputationPartition<Key, Gathered>,
}

impl<'run, Key, Gathered> WorthQueryComputationPartitionView<'run, Key, Gathered> {
    pub(super) const fn new(partition: &'run GatheredComputationPartition<Key, Gathered>) -> Self {
        Self { partition }
    }

    /// The partition's identity, derived from its key's canonical encoding.
    pub const fn identity(&self) -> PartitionIdentity {
        self.partition.identity
    }

    /// The typed key of the partition: the key of its least item, whatever
    /// order the input holds the items in.
    pub const fn key(&self) -> &'run Key {
        &self.partition.key
    }

    /// What the owner gathered for the partition.
    pub const fn gathered(&self) -> &'run Gathered {
        &self.partition.gathered
    }

    /// The identities of the partition's items in ascending order.
    pub fn items(&self) -> &'run [PartitionItemId] {
        &self.partition.items
    }
}

/// One gathered partition: the value its kernel is dispatched with.
pub(super) struct GatheredComputationPartition<Key, Gathered> {
    pub(super) identity: PartitionIdentity,
    pub(super) key: Key,
    pub(super) items: Vec<PartitionItemId>,
    pub(super) gathered: Gathered,
}

/// The item list and what the owner gathered are what a partition retains
/// beyond its inline value. A key's own heap storage is not visible here.
impl<Key, Gathered: ChargedBytes> ChargedBytes for GatheredComputationPartition<Key, Gathered> {
    fn additional_charged_bytes(&self) -> u64 {
        self.items
            .capacity()
            .checked_mul(std::mem::size_of::<PartitionItemId>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .unwrap_or(u64::MAX)
            .saturating_add(self.gathered.additional_charged_bytes())
    }
}
