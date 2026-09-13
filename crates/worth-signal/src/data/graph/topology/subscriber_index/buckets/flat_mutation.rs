use std::collections::{BTreeMap, BTreeSet};

use crate::data::handle::NodeId;
use crate::data::output::PartitionTokenId;

use super::fork_overlay::ReverseSubscriptionFlat;
use super::{
    DetailScopeKey, IndexedSubscriptionMembership, IndexedSubscriptionScope, SubscriberScopeBuckets,
};

pub(super) fn insert_flat_membership(
    flat: &mut ReverseSubscriptionFlat,
    consumer: NodeId,
    membership: &IndexedSubscriptionMembership,
) {
    let buckets = flat.buckets.entry(membership.key).or_default();
    buckets.all.insert(consumer);
    match membership.scope {
        IndexedSubscriptionScope::Unscoped => {
            buckets.unscoped.insert(consumer);
        }
        IndexedSubscriptionScope::WholePartition(partition) => {
            buckets
                .whole_partitions
                .entry(partition)
                .or_default()
                .insert(consumer);
            buckets
                .partition_scoped
                .entry(partition)
                .or_default()
                .insert(consumer);
        }
        IndexedSubscriptionScope::Detail(partition, detail) => {
            buckets
                .exact_details
                .entry(DetailScopeKey { partition, detail })
                .or_default()
                .insert(consumer);
            buckets
                .partition_scoped
                .entry(partition)
                .or_default()
                .insert(consumer);
        }
    }
}

pub(super) fn remove_flat_consumer(flat: &mut ReverseSubscriptionFlat, consumer: NodeId) {
    let Some(memberships) = flat.by_consumer.remove(&consumer) else {
        return;
    };
    for membership in memberships {
        let key = membership.key;
        let Some(buckets) = flat.buckets.get_mut(&key) else {
            continue;
        };
        buckets.all.remove(&consumer);
        match membership.scope {
            IndexedSubscriptionScope::Unscoped => {
                buckets.unscoped.remove(&consumer);
            }
            IndexedSubscriptionScope::WholePartition(partition) => {
                remove_flat_member(&mut buckets.whole_partitions, partition, consumer);
                refresh_flat_partition_scoped(buckets, partition, consumer);
            }
            IndexedSubscriptionScope::Detail(partition, detail) => {
                remove_flat_member(
                    &mut buckets.exact_details,
                    DetailScopeKey { partition, detail },
                    consumer,
                );
                refresh_flat_partition_scoped(buckets, partition, consumer);
            }
        }
        if buckets.all.is_empty() {
            flat.buckets.remove(&key);
        }
    }
}

fn refresh_flat_partition_scoped(
    buckets: &mut SubscriberScopeBuckets,
    partition: PartitionTokenId,
    consumer: NodeId,
) {
    let remains = buckets
        .whole_partitions
        .get(&partition)
        .is_some_and(|members| members.contains(&consumer))
        || buckets.exact_details.iter().any(|(candidate, members)| {
            candidate.partition == partition && members.contains(&consumer)
        });
    if !remains {
        remove_flat_member(&mut buckets.partition_scoped, partition, consumer);
    }
}

fn remove_flat_member<K: Copy + Ord>(
    buckets: &mut BTreeMap<K, BTreeSet<NodeId>>,
    key: K,
    consumer: NodeId,
) {
    let remove_key = buckets.get_mut(&key).is_some_and(|members| {
        members.remove(&consumer);
        members.is_empty()
    });
    if remove_key {
        buckets.remove(&key);
    }
}
