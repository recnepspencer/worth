use std::collections::{BTreeMap, BTreeSet};

use crate::data::handle::NodeId;
use crate::data::output::{InternedScopePath, ScopeCoverage};

use super::fork_overlay::ReverseSubscriptionFlat;
use super::{IndexedSubscriptionMembership, IndexedSubscriptionScope};

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
        IndexedSubscriptionScope::Path(path, coverage) => {
            buckets.same_path.entry(path).or_default().insert(consumer);
            if coverage == ScopeCoverage::Subtree {
                buckets
                    .subtree_covering
                    .entry(path)
                    .or_default()
                    .insert(consumer);
            }
            for depth in 1..=path.depth() {
                let prefix = path.prefix(depth).expect("bounded path prefix");
                buckets
                    .subtree_members
                    .entry(prefix)
                    .or_default()
                    .insert(consumer);
            }
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
            IndexedSubscriptionScope::Path(path, coverage) => {
                remove_member(&mut buckets.same_path, path, consumer);
                if coverage == ScopeCoverage::Subtree {
                    remove_member(&mut buckets.subtree_covering, path, consumer);
                }
                for depth in 1..=path.depth() {
                    let prefix = path.prefix(depth).expect("bounded path prefix");
                    remove_member(&mut buckets.subtree_members, prefix, consumer);
                }
            }
        }
        if buckets.all.is_empty() {
            flat.buckets.remove(&key);
        }
    }
}

fn remove_member(
    buckets: &mut BTreeMap<InternedScopePath, BTreeSet<NodeId>>,
    path: InternedScopePath,
    consumer: NodeId,
) {
    let empty = buckets.get_mut(&path).is_some_and(|members| {
        members.remove(&consumer);
        members.is_empty()
    });
    if empty {
        buckets.remove(&path);
    }
}
