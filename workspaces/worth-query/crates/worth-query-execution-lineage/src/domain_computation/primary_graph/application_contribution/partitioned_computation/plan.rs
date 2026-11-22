//! What an owner declares about an input's items, and what one partition's
//! gathering and kernel are handed.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;

/// The items an input holds.
pub struct WorthQueryComputationPartitionPlan<Item> {
    pub(crate) shape: PlanShape<Item>,
}

/// The partitioner a plan names. Keyed grouping is the only one today.
pub enum PlanShape<Item> {
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
    pub(crate) const fn new(
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
    pub(crate) const fn new(partition: &'run GatheredComputationPartition<Key, Gathered>) -> Self {
        Self { partition }
    }

    /// The partition's identity, derived from its key's canonical encoding.
    pub const fn identity(&self) -> PartitionIdentity {
        self.partition.identity
    }

    /// The typed key of the partition: the key of its least item, whatever
    /// order the input holds the items in.
    pub fn key(&self) -> &'run Key {
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
pub struct GatheredComputationPartition<Key, Gathered> {
    pub identity: PartitionIdentity,
    pub key: Arc<Key>,
    pub items: Arc<[PartitionItemId]>,
    pub gathered: Gathered,
}

/// Crate-boundary internal port for partitioned computation plans and views.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait ComputationPartitionPlanInternalPort<Item> {
    fn into_shape(self) -> PlanShape<Item>;
}

impl<Item> ComputationPartitionPlanInternalPort<Item> for WorthQueryComputationPartitionPlan<Item> {
    fn into_shape(self) -> PlanShape<Item> {
        self.shape
    }
}

pub trait ComputationPartitionMembersInternalPort<'gather, Key, Item>: Sized {
    fn from_parts(
        identity: PartitionIdentity,
        key: &'gather Key,
        members: &'gather [PartitionItemId],
        items: &'gather BTreeMap<PartitionItemId, Item>,
    ) -> Self;
}

impl<'gather, Key, Item> ComputationPartitionMembersInternalPort<'gather, Key, Item>
    for WorthQueryComputationPartitionMembers<'gather, Key, Item>
{
    fn from_parts(
        identity: PartitionIdentity,
        key: &'gather Key,
        members: &'gather [PartitionItemId],
        items: &'gather BTreeMap<PartitionItemId, Item>,
    ) -> Self {
        Self::new(identity, key, members, items)
    }
}

pub trait ComputationPartitionViewInternalPort<'run, Key, Gathered>: Sized {
    fn from_gathered(partition: &'run GatheredComputationPartition<Key, Gathered>) -> Self;
}

impl<'run, Key, Gathered> ComputationPartitionViewInternalPort<'run, Key, Gathered>
    for WorthQueryComputationPartitionView<'run, Key, Gathered>
{
    fn from_gathered(partition: &'run GatheredComputationPartition<Key, Gathered>) -> Self {
        Self::new(partition)
    }
}
