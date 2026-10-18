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

use super::super::fact::WorthQueryApplicationFactKey;

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
    fn reads(&self) -> impl Iterator<Item = ComputationRead> + '_ {
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
    handler: Option<std::sync::Arc<BTreeSet<WorthQueryApplicationFactKey>>>,
    calls: BTreeSet<ComputationRead>,
    fresh_calls: BTreeSet<ComputationRead>,
    carried: BTreeSet<ComputationRead>,
    native: Option<
        crate::domain_computation::primary_graph::application_contribution::NativeComputationReads,
    >,
}

impl ComputationFactAttribution {
    pub(in crate::domain_computation::primary_graph) fn begin_fresh(
        &mut self,
        read: ComputationRead,
    ) {
        self.calls.insert(read);
        self.fresh_calls.insert(read);
    }
    pub(in crate::domain_computation::primary_graph) fn begin_native(
        &mut self,
        native: crate::domain_computation::primary_graph::application_contribution::NativeComputationReads,
    ) {
        assert!(
            self.native.is_none(),
            "one native computation prior per projection"
        );
        self.native = Some(native);
    }
    pub(in crate::domain_computation::primary_graph) fn carry_native(
        &mut self,
        read: ComputationRead,
    ) -> bool {
        if self.native.is_none() {
            return false;
        }
        super::super::super::fact_verification_observation::carried_call_record();
        self.calls.insert(read);
        self.carried.insert(read);
        true
    }
    pub(in crate::domain_computation::primary_graph) fn materialize_carried(&mut self) {
        let Some(native) = self.native.take() else {
            return;
        };
        let calls = native.facts().call_index();
        for read in std::mem::take(&mut self.carried) {
            for key in calls.keys(read) {
                super::super::super::fact_verification_observation::carried_key_expansion();
                self.record(key.clone(), read);
            }
        }
    }
    pub(in crate::domain_computation::primary_graph) fn carried_keys(
        &self,
    ) -> impl Iterator<Item = &WorthQueryApplicationFactKey> {
        self.native
            .iter()
            .flat_map(|native| native.facts().metadata.iter())
            .inspect(|_| {
                super::super::super::fact_verification_observation::fallback_carried_metadata()
            })
            .filter(|(_, fact)| {
                fact.readers
                    .reads()
                    .any(|read| self.carried.contains(&read))
            })
            .map(|(key, _)| key)
    }
    pub(in crate::domain_computation::primary_graph) fn retain_handler(
        &mut self,
        handler: std::sync::Arc<BTreeSet<WorthQueryApplicationFactKey>>,
    ) {
        self.handler = Some(handler);
    }

    /// Conservative pre-allocation bound: each read may have its own B-tree
    /// root and key vector. Key clones reserve their declared nested payload.
    pub(super) fn index_bytes(&self) -> Option<u64> {
        use worth_execution::ChargedBytes;
        let mut bytes = (size_of::<index::ComputationFactIndex>() + 2 * size_of::<usize>()) as u64;
        bytes = bytes
            .checked_add(
                (self
                    .calls
                    .len()
                    .checked_mul(11 * size_of::<ComputationRead>() + 16 * size_of::<usize>())?)
                    as u64,
            )?
            .checked_add((self.readers.len().checked_mul(size_of::<usize>())?) as u64)?;
        if let Some(handler) = &self.handler {
            bytes = bytes.checked_add(
                (size_of::<layout::CheckedReadLayout>() + 2 * size_of::<usize>()) as u64,
            )?;
            for key in handler.iter() {
                bytes = bytes
                    .checked_add(
                        (11 * size_of::<(WorthQueryApplicationFactKey, usize)>()
                            + 16 * size_of::<usize>()) as u64,
                    )?
                    .checked_add(key.additional_charged_bytes())?;
            }
        }
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
        super::super::super::fact_verification_observation::fresh_read_key_record();
        self.begin_fresh(read);
        self.readers.entry(key).or_default().record(read);
    }

    pub(in crate::domain_computation::primary_graph) fn contains_key(
        &self,
        key: &WorthQueryApplicationFactKey,
    ) -> bool {
        self.readers.contains_key(key)
            || self.native.as_ref().is_some_and(|native| {
                native.facts().metadata.get(key).is_some_and(|fact| {
                    fact.readers
                        .reads()
                        .any(|read| self.carried.contains(&read))
                })
            })
    }

    pub(in crate::domain_computation::primary_graph) fn keys(
        &self,
    ) -> impl Iterator<Item = &WorthQueryApplicationFactKey> {
        self.readers.keys()
    }
}

/// A fact's exact position in the completed original payload, and its readers.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SealedComputationFact {
    ordinal: usize,
    readers: ComputationFactReaders,
}

/// Original pre-effect facts with checked ordinal/call correspondence.
/// Postcommit rebasing never replaces this payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct SealedComputationFacts {
    index: std::sync::Arc<index::ComputationFactIndex>,
    metadata: std::sync::Arc<BTreeMap<WorthQueryApplicationFactKey, SealedComputationFact>>,
    original: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    charged: Option<u64>,
    layout: layout::SealedReadLayout,
}

mod charged;
mod layout;
mod reuse;
pub(in crate::domain_computation::primary_graph::application_attempt::read_set) use layout::CheckedReadLayout;
pub(in crate::domain_computation::primary_graph) use reuse::NativeReadRefresh;
mod index;
mod sealed;
#[cfg(test)]
mod test_mutation;
#[cfg(test)]
mod tests;
pub(in crate::domain_computation::primary_graph) use index::ComputationFactIndex;
