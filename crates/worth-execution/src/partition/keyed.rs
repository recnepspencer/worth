use std::collections::{BTreeMap, BTreeSet};
use std::mem::size_of;

use worth_foundational::PartitionIdentity;

use super::{PartitionItemId, PartitionWork};
use crate::report::ChargedBytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyedItem<K> {
    pub item: PartitionItemId,
    pub key: K,
    pub partition: PartitionIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyedDenial {
    IdentityCollision { partition: PartitionIdentity },
    KeyIdentityChanged { partition: PartitionIdentity },
}

/// Why an admitted keyed edit changed nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyedEditDenial<Admission> {
    Keyed(KeyedDenial),
    /// The caller's memory refused the partitioner's retained bound.
    Admission(Admission),
    /// The retained bound does not fit a byte count.
    BoundOverflow,
}

/// Retained key grouping. The caller supplies a stable identity for each key;
/// different keys may never alias the same partition identity. A key is
/// retained three times, once on its item and twice for its group, and each
/// copy's heap is charged; a key's clone owns no more heap than the key.
pub struct KeyedPartitioner<K> {
    items: BTreeMap<PartitionItemId, KeyedItem<K>>,
    keys: BTreeMap<K, (PartitionIdentity, BTreeSet<PartitionItemId>)>,
    identities: BTreeMap<PartitionIdentity, K>,
    /// The heap every retained copy of a key owns.
    key_bytes: u64,
}

impl<K: Ord + Clone> Default for KeyedPartitioner<K> {
    fn default() -> Self {
        Self {
            items: BTreeMap::new(),
            keys: BTreeMap::new(),
            identities: BTreeMap::new(),
            key_bytes: 0,
        }
    }
}

impl<K: Ord + Clone + ChargedBytes> KeyedPartitioner<K> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(&self, item: PartitionItemId) -> Option<PartitionIdentity> {
        self.items.get(&item).map(|entry| entry.partition)
    }

    /// The full key owning this compact partition identity.
    pub fn key(&self, partition: PartitionIdentity) -> Option<&K> {
        self.identities.get(&partition)
    }

    pub fn members(&self, partition: PartitionIdentity) -> Option<&BTreeSet<PartitionItemId>> {
        self.identities
            .get(&partition)
            .and_then(|key| self.keys.get(key).map(|(_, members)| members))
    }

    /// Every partition that holds an item, in ascending identity order.
    pub fn partitions(&self) -> impl Iterator<Item = PartitionIdentity> + '_ {
        self.identities.keys().copied()
    }

    /// What the partitioner retains now, as `retained_bytes` counts it, or
    /// `None` when that does not fit a byte count.
    pub fn charged_bytes(&self) -> Option<u64> {
        Self::retained_bytes(self.items.len(), self.keys.len(), self.key_bytes)
    }

    /// A copy holding only the items `keep` names, each in the partition it
    /// has here: the partitioner that upserting exactly those items would
    /// leave. Before the copy is built, `admit` is offered its retained
    /// bound, counted as `upsert` counts it; a refusal builds nothing.
    /// `keep` must return the same answer on both passes. Bound counting walks
    /// retained key groups without proportional allocation; after admission,
    /// copying evaluates every item again in ascending item identity order.
    pub fn kept<Admission>(
        &self,
        mut keep: impl FnMut(PartitionItemId) -> bool,
        admit: impl FnOnce(u64) -> Result<(), Admission>,
    ) -> Result<Self, KeyedEditDenial<Admission>> {
        let (mut item_count, mut key_count, mut key_bytes) = (0_usize, 0_usize, 0_u64);
        for (_, members) in self.keys.values() {
            let mut first = true;
            for item in members {
                if !keep(*item) {
                    continue;
                }
                let entry = &self.items[item];
                item_count = item_count
                    .checked_add(1)
                    .ok_or(KeyedEditDenial::BoundOverflow)?;
                let copies = if first {
                    key_count = key_count
                        .checked_add(1)
                        .ok_or(KeyedEditDenial::BoundOverflow)?;
                    first = false;
                    3
                } else {
                    1
                };
                key_bytes = entry
                    .key
                    .additional_charged_bytes()
                    .checked_mul(copies)
                    .and_then(|bytes| key_bytes.checked_add(bytes))
                    .ok_or(KeyedEditDenial::BoundOverflow)?;
            }
        }
        let bound = Self::retained_bytes(item_count, key_count, key_bytes)
            .ok_or(KeyedEditDenial::BoundOverflow)?;
        admit(bound).map_err(KeyedEditDenial::Admission)?;
        let mut copy = Self::default();
        for entry in self.items.values().filter(|entry| keep(entry.item)) {
            let key_heap = entry.key.additional_charged_bytes();
            if !copy.keys.contains_key(&entry.key) {
                let (identity_key, group_key) = (entry.key.clone(), entry.key.clone());
                copy.key_bytes = copy
                    .key_bytes
                    .saturating_add(identity_key.additional_charged_bytes())
                    .saturating_add(group_key.additional_charged_bytes());
                copy.identities.insert(entry.partition, identity_key);
                copy.keys
                    .insert(group_key, (entry.partition, BTreeSet::new()));
            }
            if let Some((_, members)) = copy.keys.get_mut(&entry.key) {
                members.insert(entry.item);
            }
            copy.key_bytes = copy.key_bytes.saturating_add(key_heap);
            copy.items.insert(entry.item, entry.clone());
        }
        Ok(copy)
    }

    /// The most bytes the partitioner retains with `items` items in `keys`
    /// keys whose retained copies own `key_bytes` of heap, beyond its own
    /// inline value: every item's entry and membership, every key's group and
    /// identity entry, and the keys' heap.
    pub fn retained_bytes(items: usize, keys: usize, key_bytes: u64) -> Option<u64> {
        let per_item = size_of::<(PartitionItemId, KeyedItem<K>)>()
            .checked_add(size_of::<PartitionItemId>())?;
        let per_key = size_of::<(K, (PartitionIdentity, BTreeSet<PartitionItemId>))>()
            .checked_add(size_of::<(PartitionIdentity, K)>())?;
        let bytes = items
            .checked_mul(per_item)?
            .checked_add(keys.checked_mul(per_key)?)?;
        u64::try_from(bytes).ok()?.checked_add(key_bytes)
    }

    /// Routes `entry`. Before anything changes, `admit` is offered the
    /// retained bound after the edit; a refusal leaves the partitioner as it
    /// was.
    pub fn upsert<Admission>(
        &mut self,
        entry: KeyedItem<K>,
        admit: impl FnOnce(u64) -> Result<(), Admission>,
    ) -> Result<PartitionWork, KeyedEditDenial<Admission>> {
        if let Some((identity, _)) = self.keys.get(&entry.key) {
            if *identity != entry.partition {
                return Err(KeyedEditDenial::Keyed(KeyedDenial::KeyIdentityChanged {
                    partition: *identity,
                }));
            }
        }
        if self
            .identities
            .get(&entry.partition)
            .is_some_and(|key| key != &entry.key)
        {
            return Err(KeyedEditDenial::Keyed(KeyedDenial::IdentityCollision {
                partition: entry.partition,
            }));
        }
        let previous_heap = self
            .items
            .get(&entry.item)
            .map_or(0, |previous| previous.key.additional_charged_bytes());
        let key_heap = entry.key.additional_charged_bytes();
        let new_key = !self.keys.contains_key(&entry.key);
        let items = self.items.len() + usize::from(!self.items.contains_key(&entry.item));
        let keys = self.keys.len() + usize::from(new_key);
        let bound = key_heap
            .checked_mul(if new_key { 3 } else { 1 })
            .and_then(|added| {
                self.key_bytes
                    .saturating_sub(previous_heap)
                    .checked_add(added)
            })
            .and_then(|key_bytes| Self::retained_bytes(items, keys, key_bytes))
            .ok_or(KeyedEditDenial::BoundOverflow)?;
        admit(bound).map_err(KeyedEditDenial::Admission)?;
        if self.items.get(&entry.item) == Some(&entry) {
            return Ok(PartitionWork::default());
        }
        let rerouted = self
            .items
            .get(&entry.item)
            .is_none_or(|old| old.partition != entry.partition);
        self.remove(entry.item);
        if !self.keys.contains_key(&entry.key) {
            let (identity_key, group_key) = (entry.key.clone(), entry.key.clone());
            self.key_bytes = self
                .key_bytes
                .saturating_add(identity_key.additional_charged_bytes())
                .saturating_add(group_key.additional_charged_bytes());
            self.identities.insert(entry.partition, identity_key);
            self.keys
                .insert(group_key, (entry.partition, BTreeSet::new()));
        }
        if let Some((_, members)) = self.keys.get_mut(&entry.key) {
            members.insert(entry.item);
        }
        self.key_bytes = self.key_bytes.saturating_add(key_heap);
        self.items.insert(entry.item, entry);
        Ok(PartitionWork {
            items_rerouted: u64::from(rerouted),
            members_visited: 1,
            ..PartitionWork::default()
        })
    }

    /// Removes `item`; the retained bound only shrinks, so nothing is
    /// admitted.
    pub fn remove(&mut self, item: PartitionItemId) -> PartitionWork {
        let Some(entry) = self.items.remove(&item) else {
            return PartitionWork::default();
        };
        let mut released = entry.key.additional_charged_bytes();
        let Some((_, members)) = self.keys.get_mut(&entry.key) else {
            unreachable!()
        };
        members.remove(&item);
        if members.is_empty() {
            if let Some((group_key, _)) = self.keys.remove_entry(&entry.key) {
                released = released.saturating_add(group_key.additional_charged_bytes());
            }
            if let Some(identity_key) = self.identities.remove(&entry.partition) {
                released = released.saturating_add(identity_key.additional_charged_bytes());
            }
        }
        self.key_bytes = self.key_bytes.saturating_sub(released);
        PartitionWork {
            items_rerouted: 1,
            members_visited: 1,
            ..PartitionWork::default()
        }
    }
}
