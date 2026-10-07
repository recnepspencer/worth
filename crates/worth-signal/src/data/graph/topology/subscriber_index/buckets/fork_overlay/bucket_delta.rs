use std::collections::BTreeSet;

use crate::data::handle::NodeId;
use crate::data::output::{InternedScopePath, ScopeCoverage};
use crate::data::persistent_hash_map::PersistentHashMap;

use super::SetDelta;
use crate::data::graph::topology::subscriber_index::buckets::{
    IndexedSubscriptionScope, SubscriberScopeBuckets,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in super::super) struct BucketDelta {
    pub(in super::super) all: SetDelta,
    pub(in super::super) unscoped: SetDelta,
    pub(in super::super) same_path: PersistentHashMap<InternedScopePath, SetDelta>,
    pub(in super::super) subtree_covering: PersistentHashMap<InternedScopePath, SetDelta>,
    pub(in super::super) subtree_members: PersistentHashMap<InternedScopePath, SetDelta>,
}

impl Default for BucketDelta {
    fn default() -> Self {
        Self {
            all: SetDelta::default(),
            unscoped: SetDelta::default(),
            same_path: PersistentHashMap::new_persistent_overlay(),
            subtree_covering: PersistentHashMap::new_persistent_overlay(),
            subtree_members: PersistentHashMap::new_persistent_overlay(),
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
                self.unscoped.insert(base.map(|b| &b.unscoped), consumer);
            }
            IndexedSubscriptionScope::Path(path, coverage) => {
                insert_member(
                    &mut self.same_path,
                    base.and_then(|b| b.same_path.get(&path)),
                    path,
                    consumer,
                );
                if coverage == ScopeCoverage::Subtree {
                    insert_member(
                        &mut self.subtree_covering,
                        base.and_then(|b| b.subtree_covering.get(&path)),
                        path,
                        consumer,
                    );
                }
                for depth in 1..=path.depth() {
                    let prefix = path.prefix(depth).expect("bounded path prefix");
                    insert_member(
                        &mut self.subtree_members,
                        base.and_then(|b| b.subtree_members.get(&prefix)),
                        prefix,
                        consumer,
                    );
                }
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
                self.unscoped.remove(base.map(|b| &b.unscoped), consumer);
            }
            IndexedSubscriptionScope::Path(path, coverage) => {
                remove_member(
                    &mut self.same_path,
                    base.and_then(|b| b.same_path.get(&path)),
                    path,
                    consumer,
                );
                if coverage == ScopeCoverage::Subtree {
                    remove_member(
                        &mut self.subtree_covering,
                        base.and_then(|b| b.subtree_covering.get(&path)),
                        path,
                        consumer,
                    );
                }
                for depth in 1..=path.depth() {
                    let prefix = path.prefix(depth).expect("bounded path prefix");
                    remove_member(
                        &mut self.subtree_members,
                        base.and_then(|b| b.subtree_members.get(&prefix)),
                        prefix,
                        consumer,
                    );
                }
            }
        }
    }

    pub(in super::super) fn is_empty(&self) -> bool {
        self.all.is_empty()
            && self.unscoped.is_empty()
            && self.same_path.is_empty()
            && self.subtree_covering.is_empty()
            && self.subtree_members.is_empty()
    }
}

fn insert_member(
    changes: &mut PersistentHashMap<InternedScopePath, SetDelta>,
    base: Option<&BTreeSet<NodeId>>,
    path: InternedScopePath,
    consumer: NodeId,
) {
    let mut delta = changes.get(&path).cloned().unwrap_or_default();
    delta.insert(base, consumer);
    replace_delta(changes, path, delta);
}

fn remove_member(
    changes: &mut PersistentHashMap<InternedScopePath, SetDelta>,
    base: Option<&BTreeSet<NodeId>>,
    path: InternedScopePath,
    consumer: NodeId,
) {
    let mut delta = changes.get(&path).cloned().unwrap_or_default();
    delta.remove(base, consumer);
    replace_delta(changes, path, delta);
}

fn replace_delta(
    changes: &mut PersistentHashMap<InternedScopePath, SetDelta>,
    path: InternedScopePath,
    delta: SetDelta,
) {
    if delta.is_empty() {
        changes.remove(&path);
    } else {
        changes.insert(path, delta);
    }
}
