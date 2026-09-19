use std::collections::{BTreeMap, BTreeSet};

use crate::data::handle::NodeId;
use crate::data::output::PartitionTokenId;
use crate::data::persistent_hash_map::PersistentHashMap;

use super::SetDelta;
use crate::data::graph::topology::subscriber_index::buckets::{
    DetailScopeKey, IndexedSubscriptionScope, SubscriberScopeBuckets,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in super::super) struct BucketDelta {
    pub(in super::super) all: SetDelta,
    pub(in super::super) unscoped: SetDelta,
    pub(in super::super) whole_partitions: PersistentHashMap<PartitionTokenId, SetDelta>,
    pub(in super::super) exact_details: PersistentHashMap<DetailScopeKey, SetDelta>,
    pub(in super::super) partition_scoped: PersistentHashMap<PartitionTokenId, SetDelta>,
}

impl Default for BucketDelta {
    fn default() -> Self {
        Self {
            all: SetDelta::default(),
            unscoped: SetDelta::default(),
            whole_partitions: PersistentHashMap::new_persistent_overlay(),
            exact_details: PersistentHashMap::new_persistent_overlay(),
            partition_scoped: PersistentHashMap::new_persistent_overlay(),
        }
    }
}

impl BucketDelta {
    pub(in super::super) fn insert(
        &mut self,
        base: Option<&SubscriberScopeBuckets>,
        consumer: NodeId,
        scope: &IndexedSubscriptionScope,
    ) {
        self.all.insert(base.map(|b| &b.all), consumer);
        match *scope {
            IndexedSubscriptionScope::Unscoped => {
                self.unscoped.insert(base.map(|b| &b.unscoped), consumer)
            }
            IndexedSubscriptionScope::WholePartition(partition) => {
                insert_map_member(
                    &mut self.whole_partitions,
                    base.and_then(|b| b.whole_partitions.get(&partition)),
                    partition,
                    consumer,
                );
                insert_map_member(
                    &mut self.partition_scoped,
                    base.and_then(|b| b.partition_scoped.get(&partition)),
                    partition,
                    consumer,
                );
            }
            IndexedSubscriptionScope::Detail(partition, detail) => {
                let key = DetailScopeKey { partition, detail };
                insert_map_member(
                    &mut self.exact_details,
                    base.and_then(|b| b.exact_details.get(&key)),
                    key,
                    consumer,
                );
                insert_map_member(
                    &mut self.partition_scoped,
                    base.and_then(|b| b.partition_scoped.get(&partition)),
                    partition,
                    consumer,
                );
            }
        }
    }

    pub(in super::super) fn remove(
        &mut self,
        base: Option<&SubscriberScopeBuckets>,
        consumer: NodeId,
        scope: &IndexedSubscriptionScope,
    ) {
        self.all.remove(base.map(|b| &b.all), consumer);
        match *scope {
            IndexedSubscriptionScope::Unscoped => {
                self.unscoped.remove(base.map(|b| &b.unscoped), consumer)
            }
            IndexedSubscriptionScope::WholePartition(partition) => {
                remove_map_member(
                    &mut self.whole_partitions,
                    base.and_then(|b| b.whole_partitions.get(&partition)),
                    partition,
                    consumer,
                );
                self.refresh_partition_scoped(base, partition, consumer);
            }
            IndexedSubscriptionScope::Detail(partition, detail) => {
                let key = DetailScopeKey { partition, detail };
                remove_map_member(
                    &mut self.exact_details,
                    base.and_then(|b| b.exact_details.get(&key)),
                    key,
                    consumer,
                );
                self.refresh_partition_scoped(base, partition, consumer);
            }
        }
    }

    pub(in super::super) fn is_empty(&self) -> bool {
        self.all.is_empty()
            && self.unscoped.is_empty()
            && self.whole_partitions.is_empty()
            && self.exact_details.is_empty()
            && self.partition_scoped.is_empty()
    }

    fn refresh_partition_scoped(
        &mut self,
        base: Option<&SubscriberScopeBuckets>,
        partition: PartitionTokenId,
        consumer: NodeId,
    ) {
        let remains = map_contains(
            base.and_then(|b| b.whole_partitions.get(&partition)),
            self.whole_partitions.get(&partition),
            &consumer,
        ) || merged_map_any(
            base.map(|b| &b.exact_details),
            &self.exact_details,
            |key| key.partition == partition,
            &consumer,
        );
        if !remains {
            remove_map_member(
                &mut self.partition_scoped,
                base.and_then(|b| b.partition_scoped.get(&partition)),
                partition,
                consumer,
            );
        }
    }
}

fn insert_map_member<K: Copy + Eq + std::hash::Hash>(
    changes: &mut PersistentHashMap<K, SetDelta>,
    base: Option<&BTreeSet<NodeId>>,
    key: K,
    consumer: NodeId,
) {
    let mut delta = changes.get(&key).cloned().unwrap_or_default();
    delta.insert(base, consumer);
    replace_delta(changes, key, delta);
}

fn remove_map_member<K: Copy + Eq + std::hash::Hash>(
    changes: &mut PersistentHashMap<K, SetDelta>,
    base: Option<&BTreeSet<NodeId>>,
    key: K,
    consumer: NodeId,
) {
    let mut delta = changes.get(&key).cloned().unwrap_or_default();
    delta.remove(base, consumer);
    replace_delta(changes, key, delta);
}

fn replace_delta<K: Copy + Eq + std::hash::Hash>(
    changes: &mut PersistentHashMap<K, SetDelta>,
    key: K,
    delta: SetDelta,
) {
    if delta.is_empty() {
        changes.remove(&key);
    } else {
        changes.insert(key, delta);
    }
}

fn map_contains(
    base: Option<&BTreeSet<NodeId>>,
    delta: Option<&SetDelta>,
    consumer: &NodeId,
) -> bool {
    delta.map_or_else(
        || base.is_some_and(|set| set.contains(consumer)),
        |delta| delta.contains(base.is_some_and(|set| set.contains(consumer)), consumer),
    )
}

fn merged_map_any<K: Copy + Ord + std::hash::Hash>(
    base: Option<&BTreeMap<K, BTreeSet<NodeId>>>,
    changes: &PersistentHashMap<K, SetDelta>,
    mut key_matches: impl FnMut(K) -> bool,
    consumer: &NodeId,
) -> bool {
    base.into_iter()
        .flatten()
        .any(|(key, set)| key_matches(*key) && map_contains(Some(set), changes.get(key), consumer))
        || changes.iter().any(|(key, delta)| {
            base.is_none_or(|base| !base.contains_key(key))
                && key_matches(*key)
                && delta.contains(false, consumer)
        })
}
