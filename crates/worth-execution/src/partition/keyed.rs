use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::PartitionIdentity;

use super::{PartitionItemId, PartitionRoute, PartitionWork, SourceFactId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyedItem<K> {
    pub item: PartitionItemId,
    pub source_fact: SourceFactId,
    pub key: K,
    pub partition: PartitionIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyedDenial {
    IdentityCollision { partition: PartitionIdentity },
    KeyIdentityChanged { partition: PartitionIdentity },
}

/// Retained key grouping. The caller supplies a stable identity for each key;
/// different keys may never alias the same partition identity.
pub struct KeyedPartitioner<K> {
    items: BTreeMap<PartitionItemId, KeyedItem<K>>,
    keys: BTreeMap<K, (PartitionIdentity, BTreeSet<PartitionItemId>)>,
    identities: BTreeMap<PartitionIdentity, K>,
}

impl<K: Ord + Clone> Default for KeyedPartitioner<K> {
    fn default() -> Self {
        Self {
            items: BTreeMap::new(),
            keys: BTreeMap::new(),
            identities: BTreeMap::new(),
        }
    }
}

impl<K: Ord + Clone> KeyedPartitioner<K> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(&self, item: PartitionItemId) -> Option<PartitionRoute> {
        self.items.get(&item).map(|entry| PartitionRoute {
            partition: entry.partition,
            source_fact: entry.source_fact,
        })
    }

    pub fn members(&self, partition: PartitionIdentity) -> Option<&BTreeSet<PartitionItemId>> {
        self.identities
            .get(&partition)
            .and_then(|key| self.keys.get(key).map(|(_, members)| members))
    }

    pub fn upsert(&mut self, entry: KeyedItem<K>) -> Result<PartitionWork, KeyedDenial> {
        if let Some((identity, _)) = self.keys.get(&entry.key) {
            if *identity != entry.partition {
                return Err(KeyedDenial::KeyIdentityChanged {
                    partition: *identity,
                });
            }
        }
        if self
            .identities
            .get(&entry.partition)
            .is_some_and(|key| key != &entry.key)
        {
            return Err(KeyedDenial::IdentityCollision {
                partition: entry.partition,
            });
        }
        if self.items.get(&entry.item) == Some(&entry) {
            return Ok(PartitionWork::default());
        }
        if self.items.get(&entry.item).is_some_and(|previous| {
            previous.key == entry.key && previous.partition == entry.partition
        }) {
            self.items.insert(entry.item, entry);
            return Ok(PartitionWork {
                members_visited: 1,
                ..PartitionWork::default()
            });
        }
        let rerouted = self
            .items
            .get(&entry.item)
            .is_none_or(|old| old.partition != entry.partition);
        self.remove(entry.item);
        self.identities.insert(entry.partition, entry.key.clone());
        self.keys
            .entry(entry.key.clone())
            .or_insert_with(|| (entry.partition, BTreeSet::new()))
            .1
            .insert(entry.item);
        self.items.insert(entry.item, entry);
        Ok(PartitionWork {
            items_rerouted: u64::from(rerouted),
            members_visited: 1,
            ..PartitionWork::default()
        })
    }

    pub fn remove(&mut self, item: PartitionItemId) -> PartitionWork {
        let Some(entry) = self.items.remove(&item) else {
            return PartitionWork::default();
        };
        let Some((_, members)) = self.keys.get_mut(&entry.key) else {
            unreachable!()
        };
        members.remove(&item);
        if members.is_empty() {
            self.keys.remove(&entry.key);
            self.identities.remove(&entry.partition);
        }
        PartitionWork {
            items_rerouted: 1,
            members_visited: 1,
            ..PartitionWork::default()
        }
    }
}
