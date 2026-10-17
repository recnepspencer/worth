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
use worth_proof::CanonicalUniqueVec;

use super::super::fact::{WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact};

/// One owner call of a partitioned computation that reads through the lent
/// reader.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) enum ComputationRead {
    /// `partitions`: which items the input holds.
    Membership,
    /// `partition_key` for one item.
    ItemKey(PartitionItemId),
    /// `gather` for one partition.
    Partition(PartitionIdentity),
}

/// Every owner call that read one decision fact while the attempt runs. A
/// call that reads the fact again is still one call.
#[derive(Default)]
struct ComputationFactReads {
    membership: bool,
    item_keys: BTreeSet<PartitionItemId>,
    partitions: BTreeSet<PartitionIdentity>,
}

impl ComputationFactReads {
    fn record(&mut self, read: ComputationRead) {
        match read {
            ComputationRead::Membership => self.membership = true,
            ComputationRead::ItemKey(item) => {
                self.item_keys.insert(item);
            }
            ComputationRead::Partition(partition) => {
                self.partitions.insert(partition);
            }
        }
    }

    fn in_order(self) -> ComputationFactReaders {
        ComputationFactReaders {
            membership: self.membership,
            item_keys: CanonicalUniqueVec::from_btree_set(self.item_keys),
            partitions: CanonicalUniqueVec::from_btree_set(self.partitions),
        }
    }
}

/// Every owner call that read one decision fact. Items and partitions are in
/// ascending order and named once, by type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ComputationFactReaders {
    membership: bool,
    item_keys: CanonicalUniqueVec<PartitionItemId>,
    partitions: CanonicalUniqueVec<PartitionIdentity>,
}

impl ComputationFactReaders {
    /// The partitions whose gathering read the fact.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn partitions(&self) -> &[PartitionIdentity] {
        self.partitions.as_slice()
    }

    /// Every call that read the fact.
    pub(in crate::domain_computation::primary_graph) fn reads(
        &self,
    ) -> impl Iterator<Item = ComputationRead> + '_ {
        self.membership
            .then_some(ComputationRead::Membership)
            .into_iter()
            .chain(
                self.item_keys
                    .as_slice()
                    .iter()
                    .copied()
                    .map(ComputationRead::ItemKey),
            )
            .chain(
                self.partitions
                    .as_slice()
                    .iter()
                    .copied()
                    .map(ComputationRead::Partition),
            )
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn read_by(
        membership: bool,
        item_keys: impl IntoIterator<Item = u64>,
        partitions: impl IntoIterator<Item = PartitionIdentity>,
    ) -> Self {
        ComputationFactReads {
            membership,
            item_keys: item_keys.into_iter().map(PartitionItemId).collect(),
            partitions: partitions.into_iter().collect(),
        }
        .in_order()
    }
}

/// The fact keys the owner calls of one attempt read, each with every call
/// that read it.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct ComputationFactAttribution {
    readers: BTreeMap<WorthQueryApplicationFactKey, ComputationFactReads>,
}

impl ComputationFactAttribution {
    /// Conservative pre-allocation bound: each read may have its own B-tree
    /// root and key vector. Key clones reserve their declared nested payload.
    pub(super) fn index_bytes(&self) -> Option<u64> {
        use worth_execution::ChargedBytes;
        let mut bytes = (size_of::<index::ComputationFactIndex>() + 2 * size_of::<usize>()) as u64;
        for (key, reads) in &self.readers {
            let key_bytes = (size_of::<WorthQueryApplicationFactKey>() as u64)
                .checked_add(key.additional_charged_bytes())?;
            // A B-tree node holds eleven slots and up to twelve pointers.
            let ordinal = 11_u64
                .checked_mul(key_bytes.checked_add(size_of::<usize>() as u64)?)?
                .checked_add(16 * size_of::<usize>() as u64)?;
            bytes = bytes.checked_add(ordinal)?;
            let count = usize::from(reads.membership)
                .checked_add(reads.item_keys.len())?
                .checked_add(reads.partitions.len())?;
            let per_read = 11_u64
                .checked_mul(
                    (size_of::<ComputationRead>() + size_of::<Vec<WorthQueryApplicationFactKey>>())
                        as u64,
                )?
                .checked_add(16 * size_of::<usize>() as u64)?
                .checked_add(4_u64.checked_mul(key_bytes)?)?;
            bytes = bytes.checked_add(per_read.checked_mul(count as u64)?)?;
        }
        Some(bytes)
    }

    /// Records that `read` read `key`. A key an earlier call already read
    /// gains this call too.
    pub(in crate::domain_computation::primary_graph) fn record(
        &mut self,
        key: WorthQueryApplicationFactKey,
        read: ComputationRead,
    ) {
        self.readers.entry(key).or_default().record(read);
    }

    pub(in crate::domain_computation::primary_graph) fn contains_key(
        &self,
        key: &WorthQueryApplicationFactKey,
    ) -> bool {
        self.readers.contains_key(key)
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
    index: std::sync::Arc<index::ComputationFactIndex>,
    facts: BTreeMap<WorthQueryApplicationFactKey, SealedComputationFact>,
}

impl SealedComputationFacts {
    /// Takes, for every key an owner call read, the fact seal observed for it.
    /// Seal observed every expected key, and every key a call read is one.
    pub(super) fn at_seal(
        attribution: ComputationFactAttribution,
        sealed: &BTreeMap<WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact>,
    ) -> Self {
        let facts = attribution
            .readers
            .into_iter()
            .map(|(key, readers)| {
                let fact = sealed
                    .get(&key)
                    .expect("seal observed every owner key")
                    .clone();
                (
                    key,
                    SealedComputationFact {
                        fact,
                        readers: readers.in_order(),
                    },
                )
            })
            .collect();
        let index = std::sync::Arc::new(index::ComputationFactIndex::build(&facts, sealed.keys()));
        Self { facts, index }
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

    /// Files a copy of what seal observed for `key` under `to`, as if the
    /// calls that read it had also read that key.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn copy_key(
        &mut self,
        key: &WorthQueryApplicationFactKey,
        to: WorthQueryApplicationFactKey,
    ) {
        let sealed = self
            .facts
            .get(key)
            .expect("an owner call read the key")
            .clone();
        self.facts.insert(to, sealed);
        self.index = std::sync::Arc::new(index::ComputationFactIndex::build(
            &self.facts,
            self.facts.keys(),
        ));
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
        self.index = std::sync::Arc::new(index::ComputationFactIndex::build(
            &self.facts,
            self.facts.keys(),
        ));
    }

    /// The most work observing every fact the calls read can take, or
    /// `None` when a fact's observation has no bound or the sum overflows.
    pub(in crate::domain_computation::primary_graph) fn observation_work_bound(
        &self,
    ) -> Option<u64> {
        self.index.observation_work
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

mod index;
pub(in crate::domain_computation::primary_graph) use index::ComputationFactIndex;
impl SealedComputationFacts {
    pub(in crate::domain_computation::primary_graph) fn call_index(
        &self,
    ) -> std::sync::Arc<ComputationFactIndex> {
        std::sync::Arc::clone(&self.index)
    }
    pub(in crate::domain_computation::primary_graph) fn at_ordinal(
        &self,
        ordinal: usize,
    ) -> Option<(
        &WorthQueryApplicationFactKey,
        &WorthQueryApplicationObservedFact,
        &ComputationFactReaders,
    )> {
        let key = self.index.ordinals.get(&ordinal)?;
        let value = self.facts.get(key)?;
        Some((key, &value.fact, &value.readers))
    }
}
