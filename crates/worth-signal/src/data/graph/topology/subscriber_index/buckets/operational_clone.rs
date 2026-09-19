use std::collections::{BTreeMap, BTreeSet};

use crate::data::handle::NodeId;
use crate::data::persistent_hash_map::PersistentHashMap;

use super::fork_overlay::{
    extend_merged_set, BucketDelta, ReverseSubscriptionFlat, ReverseSubscriptionStorage, SetDelta,
};
use super::{ForkConsumerMembershipChange, ReverseSubscriptionIndex, SubscriberScopeBuckets};

impl ReverseSubscriptionIndex {
    pub(crate) fn operational_clone(&self) -> Self {
        let flat = match &self.storage {
            ReverseSubscriptionStorage::Exclusive(flat) => flat.clone(),
            ReverseSubscriptionStorage::ForkShared {
                base,
                bucket_changes,
                consumer_changes,
            } => materialize_flat(base, bucket_changes, consumer_changes),
        };
        Self {
            storage: ReverseSubscriptionStorage::Exclusive(flat),
            valid: self.valid,
        }
    }
}

fn materialize_flat(
    base: &ReverseSubscriptionFlat,
    bucket_changes: &PersistentHashMap<super::ProducerAspectKey, BucketDelta>,
    consumer_changes: &PersistentHashMap<NodeId, ForkConsumerMembershipChange>,
) -> ReverseSubscriptionFlat {
    let mut bucket_keys = base.buckets.keys().copied().collect::<BTreeSet<_>>();
    bucket_keys.extend(bucket_changes.iter().map(|(key, _)| *key));
    let mut buckets = BTreeMap::new();
    for key in bucket_keys {
        let values = materialize_bucket(base.buckets.get(&key), bucket_changes.get(&key));
        if !values.all.is_empty() {
            buckets.insert(key, values);
        }
    }

    let mut by_consumer = base.by_consumer.clone();
    for (consumer, change) in consumer_changes.iter() {
        match change {
            ForkConsumerMembershipChange::Replaced(memberships) => {
                by_consumer.insert(*consumer, memberships.to_owned());
            }
            ForkConsumerMembershipChange::Removed => {
                by_consumer.remove(consumer);
            }
        }
    }
    ReverseSubscriptionFlat {
        buckets,
        by_consumer,
    }
}

fn materialize_bucket(
    base: Option<&SubscriberScopeBuckets>,
    delta: Option<&BucketDelta>,
) -> SubscriberScopeBuckets {
    SubscriberScopeBuckets {
        all: materialize_set(base.map(|b| &b.all), delta.map(|d| &d.all)),
        unscoped: materialize_set(base.map(|b| &b.unscoped), delta.map(|d| &d.unscoped)),
        whole_partitions: materialize_map(
            base.map(|b| &b.whole_partitions),
            delta.map(|d| &d.whole_partitions),
        ),
        exact_details: materialize_map(
            base.map(|b| &b.exact_details),
            delta.map(|d| &d.exact_details),
        ),
        partition_scoped: materialize_map(
            base.map(|b| &b.partition_scoped),
            delta.map(|d| &d.partition_scoped),
        ),
    }
}

fn materialize_map<K: Copy + Ord + std::hash::Hash>(
    base: Option<&BTreeMap<K, BTreeSet<NodeId>>>,
    changes: Option<&PersistentHashMap<K, SetDelta>>,
) -> BTreeMap<K, BTreeSet<NodeId>> {
    let mut keys = base
        .into_iter()
        .flatten()
        .map(|(key, _)| *key)
        .collect::<BTreeSet<_>>();
    if let Some(changes) = changes {
        keys.extend(changes.iter().map(|(key, _)| *key));
    }
    keys.into_iter()
        .filter_map(|key| {
            let values = materialize_set(
                base.and_then(|base| base.get(&key)),
                changes.and_then(|changes| changes.get(&key)),
            );
            (!values.is_empty()).then_some((key, values))
        })
        .collect()
}

fn materialize_set(base: Option<&BTreeSet<NodeId>>, delta: Option<&SetDelta>) -> BTreeSet<NodeId> {
    let mut values = Vec::new();
    let _ = extend_merged_set(
        base,
        delta,
        &mut values,
        &mut crate::logic::evaluation::EvaluationWork::Ordinary,
    )
    .expect("materializable reverse subscription set");
    values.into_iter().collect()
}
