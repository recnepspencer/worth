//! Which reads of a partitioned computation read each decision fact.
//!
//! A handler that runs a partitioned computation lends it the operation's
//! reader. Every fact key an owner call reads is recorded with that call, so a
//! key two partitions read belongs to both. At seal the keys become ordinals
//! of the sealed facts and the routing table is built once.

use std::collections::{BTreeMap, BTreeSet};

use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;

use super::super::fact::WorthQueryApplicationFactKey;

/// One owner call of a partitioned computation that reads through the lent
/// reader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum ComputationRead {
    /// `partitions`: which items the input holds.
    Membership,
    /// `partition_key` for one item.
    ItemKey(PartitionItemId),
    /// `gather` for one partition.
    Partition(PartitionIdentity),
}

/// What read one decision fact.
///
/// A fact read in more than one class belongs to the first of them: the
/// handler, the membership, the keys of items, the partitions. Items and
/// partitions are in ascending order and named once.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum ComputationFactReaders {
    Handler,
    Membership,
    ItemKeys(Vec<PartitionItemId>),
    Partitions(Vec<PartitionIdentity>),
}

impl ComputationFactReaders {
    fn of(read: ComputationRead) -> Self {
        match read {
            ComputationRead::Membership => Self::Membership,
            ComputationRead::ItemKey(item) => Self::ItemKeys(vec![item]),
            ComputationRead::Partition(partition) => Self::Partitions(vec![partition]),
        }
    }

    fn also_read_in(&mut self, read: ComputationRead) {
        match (&mut *self, read) {
            (Self::Handler | Self::Membership, _)
            | (Self::ItemKeys(_), ComputationRead::Partition(_)) => {}
            (_, ComputationRead::Membership) => *self = Self::Membership,
            (Self::ItemKeys(items), ComputationRead::ItemKey(item)) => items.push(item),
            (Self::Partitions(_), ComputationRead::ItemKey(_)) => *self = Self::of(read),
            (Self::Partitions(partitions), ComputationRead::Partition(partition)) => {
                partitions.push(partition);
            }
        }
    }

    fn in_order(mut self) -> Self {
        match &mut self {
            Self::Handler | Self::Membership => {}
            Self::ItemKeys(items) => {
                items.sort_unstable();
                items.dedup();
            }
            Self::Partitions(partitions) => {
                partitions.sort_unstable();
                partitions.dedup();
            }
        }
        self
    }
}

/// The fact keys the owner calls of one attempt read, each with every call
/// that read it. A key it does not hold was read by the handler alone.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct ComputationFactAttribution {
    readers: BTreeMap<WorthQueryApplicationFactKey, ComputationFactReaders>,
}

impl ComputationFactAttribution {
    /// Records that `read` read `key`. A key an earlier call already read
    /// gains this call too.
    pub(in crate::domain_computation::primary_graph) fn record(
        &mut self,
        key: WorthQueryApplicationFactKey,
        read: ComputationRead,
    ) {
        match self.readers.get_mut(&key) {
            Some(readers) => readers.also_read_in(read),
            None => {
                self.readers.insert(key, ComputationFactReaders::of(read));
            }
        }
    }

    /// Gives the handler every key it read itself, whatever else read it.
    pub(in crate::domain_computation::primary_graph) fn yield_to_handler(
        &mut self,
        handler: &BTreeSet<WorthQueryApplicationFactKey>,
    ) {
        self.readers.retain(|key, _| !handler.contains(key));
    }

    pub(in crate::domain_computation::primary_graph) fn keys(
        &self,
    ) -> impl Iterator<Item = &WorthQueryApplicationFactKey> {
        self.readers.keys()
    }
}

/// The routing table of one sealed read set: what read each handler fact, by
/// the fact's ordinal in the sealed fact sequence.
#[derive(Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ComputationFactRouting {
    readers: Box<[ComputationFactReaders]>,
}

impl ComputationFactRouting {
    /// Builds the table over the sealed handler facts, whose order is their
    /// ordinals.
    pub(super) fn at_seal<'key>(
        mut attribution: ComputationFactAttribution,
        handler_facts: impl Iterator<Item = &'key WorthQueryApplicationFactKey>,
    ) -> Self {
        Self {
            readers: handler_facts
                .map(|key| {
                    attribution
                        .readers
                        .remove(key)
                        .map_or(ComputationFactReaders::Handler, |readers| {
                            readers.in_order()
                        })
                })
                .collect(),
        }
    }

    /// What read each handler fact, by ordinal.
    // Nothing routes a mark yet: tests read the table until retention does.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::domain_computation::primary_graph) fn readers(
        &self,
    ) -> &[ComputationFactReaders] {
        &self.readers
    }
}

#[cfg(test)]
mod tests;
