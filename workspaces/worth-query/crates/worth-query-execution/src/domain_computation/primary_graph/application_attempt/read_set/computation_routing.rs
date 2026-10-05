//! Which owner calls of a partitioned computation read each decision fact,
//! and the facts they read as the attempt sealed them.
//!
//! A handler that runs a partitioned computation lends it the operation's
//! reader. Every fact key an owner call reads is recorded with that call, so a
//! key two partitions read belongs to both, and a key the membership, an
//! item's key and a partition read belongs to all three. The handler is not
//! one of them: it runs again on every attempt and reads its own facts itself,
//! so a key it read beside an owner call is still that call's. At seal every
//! key keeps the fact the attempt observed for it, before its own effect.

use std::collections::{BTreeMap, BTreeSet};

use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;

use super::super::fact::{WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact};

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

/// Every owner call that read one decision fact. Items and partitions are in
/// ascending order and named once.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ComputationFactReaders {
    membership: bool,
    item_keys: Vec<PartitionItemId>,
    partitions: Vec<PartitionIdentity>,
}

impl ComputationFactReaders {
    fn record(&mut self, read: ComputationRead) {
        match read {
            ComputationRead::Membership => self.membership = true,
            ComputationRead::ItemKey(item) => self.item_keys.push(item),
            ComputationRead::Partition(partition) => self.partitions.push(partition),
        }
    }

    fn in_order(mut self) -> Self {
        self.item_keys.sort_unstable();
        self.item_keys.dedup();
        self.partitions.sort_unstable();
        self.partitions.dedup();
        self
    }

    /// Whether the membership or an item's key read the fact: a change to it
    /// can change which partitions there are.
    pub(in crate::domain_computation::primary_graph) fn partitioner(&self) -> bool {
        self.membership || !self.item_keys.is_empty()
    }

    /// The partitions whose gathering read the fact.
    pub(in crate::domain_computation::primary_graph) fn partitions(&self) -> &[PartitionIdentity] {
        &self.partitions
    }

    /// Every call that read the fact.
    pub(in crate::domain_computation::primary_graph) fn reads(
        &self,
    ) -> impl Iterator<Item = ComputationRead> + '_ {
        self.membership
            .then_some(ComputationRead::Membership)
            .into_iter()
            .chain(self.item_keys.iter().copied().map(ComputationRead::ItemKey))
            .chain(
                self.partitions
                    .iter()
                    .copied()
                    .map(ComputationRead::Partition),
            )
    }

    /// Whether a call a run carried read the fact: the membership, an item's
    /// key, or a gathering of one of the `skipped` partitions.
    pub(in crate::domain_computation::primary_graph) fn read_by_carried(
        &self,
        skipped: &BTreeSet<PartitionIdentity>,
    ) -> bool {
        self.partitioner()
            || self
                .partitions
                .iter()
                .any(|partition| skipped.contains(partition))
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn read_by(
        membership: bool,
        item_keys: impl IntoIterator<Item = u64>,
        partitions: impl IntoIterator<Item = PartitionIdentity>,
    ) -> Self {
        Self {
            membership,
            item_keys: item_keys.into_iter().map(PartitionItemId).collect(),
            partitions: partitions.into_iter().collect(),
        }
    }
}

/// The fact keys the owner calls of one attempt read, each with every call
/// that read it.
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
        self.readers.entry(key).or_default().record(read);
    }

    pub(in crate::domain_computation::primary_graph) fn keys(
        &self,
    ) -> impl Iterator<Item = &WorthQueryApplicationFactKey> {
        self.readers.keys()
    }
}

/// One fact an owner call read, as the attempt observed it before its own
/// effect, with every call that read it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct SealedComputationFact {
    fact: WorthQueryApplicationObservedFact,
    readers: ComputationFactReaders,
}

/// Every fact the owner calls of one attempt read, by key, as sealed.
///
/// These are the facts themselves, not the record's: a record's facts are
/// rebased to source revisions at commit, so they cannot be compared by
/// content and they hide the attempt's own effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct SealedComputationFacts {
    facts: BTreeMap<WorthQueryApplicationFactKey, SealedComputationFact>,
}

impl SealedComputationFacts {
    /// Takes, for every key an owner call read, the fact seal observed for it.
    /// Seal observed every expected key, and every key a call read is one.
    pub(super) fn at_seal(
        attribution: ComputationFactAttribution,
        sealed: &BTreeMap<WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact>,
    ) -> Self {
        Self {
            facts: attribution
                .readers
                .into_iter()
                .map(|(key, readers)| {
                    let fact = sealed
                        .get(&key)
                        .expect("seal observed every key an owner call read")
                        .clone();
                    let readers = readers.in_order();
                    (key, SealedComputationFact { fact, readers })
                })
                .collect(),
        }
    }

    /// The fact seal observed for `key`, when an owner call read it.
    pub(in crate::domain_computation::primary_graph) fn fact(
        &self,
        key: &WorthQueryApplicationFactKey,
    ) -> Option<&WorthQueryApplicationObservedFact> {
        self.facts.get(key).map(|sealed| &sealed.fact)
    }

    /// Stands `fact` in for what seal observed for `key`, as if it had moved
    /// since.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn replace_fact(
        &mut self,
        key: &WorthQueryApplicationFactKey,
        fact: WorthQueryApplicationObservedFact,
    ) {
        self.facts
            .get_mut(key)
            .expect("an owner call read the key")
            .fact = fact;
    }

    /// Files what seal observed for `key` under `to`, as if a call had read
    /// that key instead.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn rename_key(
        &mut self,
        key: &WorthQueryApplicationFactKey,
        to: WorthQueryApplicationFactKey,
    ) {
        let sealed = self.facts.remove(key).expect("an owner call read the key");
        self.facts.insert(to, sealed);
    }

    /// Each fact a call read, in key order, with every call that read it.
    pub(in crate::domain_computation::primary_graph) fn facts(
        &self,
    ) -> impl Iterator<
        Item = (
            &WorthQueryApplicationFactKey,
            &WorthQueryApplicationObservedFact,
            &ComputationFactReaders,
        ),
    > {
        self.facts
            .iter()
            .map(|(key, sealed)| (key, &sealed.fact, &sealed.readers))
    }
}

mod charged;
#[cfg(test)]
mod tests;
