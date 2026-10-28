//! Closed ownership over the same funded AVL cells. No root can escape.
use super::{
    node::{self, Link, Node, Rotation},
    ExecutionOrderedMap, ExecutionOrderedMapDenial,
};
use crate::{ExecutionAllocationPolicy, ExecutionRequest};
use std::{cmp::Ordering, sync::Arc};

/// Ordered storage with allocation-free removal, including after its original
/// request stops. Inserts use the existing funded AVL editor. This mode cannot
/// be cloned, converted from shared roots or borrowed as a shared map; values
/// and iterators borrow entries without sharing any tree node.
///
/// ```compile_fail,E0599
/// let map = worth_execution::ExecutionUniqueOrderedMap::<u64, u64>::default();
/// let shared = map.clone();
/// ```
pub struct ExecutionUniqueOrderedMap<K, V> {
    map: ExecutionOrderedMap<K, V>,
}

impl<K, V> Default for ExecutionUniqueOrderedMap<K, V> {
    fn default() -> Self {
        Self {
            map: ExecutionOrderedMap::default(),
        }
    }
}

impl<K, V> ExecutionUniqueOrderedMap<K, V> {
    /// Number of retained entries.
    pub fn len(&self) -> usize {
        self.map.len()
    }
    /// Whether no entry is retained.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    /// Actual retained cell quotes, or `None` for mixed unaccounted backing.
    pub fn accounted_bytes(&self) -> Option<u64> {
        self.map.accounted_bytes()
    }
    /// Borrow entries in key order without sharing any tree node.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&K, &V)> {
        self.map.iter()
    }
    /// Borrow values in key order without sharing any tree node.
    pub fn values(&self) -> impl ExactSizeIterator<Item = &V> {
        self.map.values()
    }
    /// Borrow keys in their canonical order without sharing any tree node.
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &K> {
        self.map.keys()
    }
}

impl<K: Ord, V> ExecutionUniqueOrderedMap<K, V> {
    /// Borrow the value selected by an exact key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    /// No allocation, admission, clone of a payload, or new payer. Original
    /// grants remain with their cells through rotations and release on drop.
    pub fn remove(&mut self, key: &K) -> bool {
        remove(&mut self.map.root, key)
    }
}

impl<K: Ord + Clone, V: Clone> ExecutionUniqueOrderedMap<K, V> {
    /// Native payloads retain their own backing; this admits every new entry
    /// and copied path cell before allocation through the original request.
    pub fn insert_cloned_native_for_request(
        &mut self,
        key: &K,
        value: &V,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<(), ExecutionOrderedMapDenial> {
        self.map
            .insert_cloned_native_for_request(key, value, request)
    }
    /// Retains the existing policy door for owners already funded that way.
    pub fn insert_cloned_native_in(
        &mut self,
        key: &K,
        value: &V,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), ExecutionOrderedMapDenial> {
        self.map.insert_cloned_native_in(key, value, policy)
    }
}

fn owned<K, V>(link: &mut Link<K, V>) -> &mut Node<K, V> {
    Arc::get_mut(link.as_mut().expect("owned node")).expect("the closed map never shares a node")
}

fn remove<K: Ord, V>(link: &mut Link<K, V>, key: &K) -> bool {
    let Some(node) = link.as_ref() else {
        return false;
    };
    let order = key.cmp(&node.entry.key);
    let removed = match order {
        Ordering::Less => remove(&mut owned(link).left, key),
        Ordering::Greater => remove(&mut owned(link).right, key),
        Ordering::Equal => {
            if node.left.is_none() || node.right.is_none() {
                let node = owned(link);
                let child = node.left.take().or_else(|| node.right.take());
                *link = child;
                return true;
            }
            let node = owned(link);
            let mut first = node.right.as_ref().expect("right child");
            while let Some(left) = &first.left {
                first = left;
            }
            node.entry = Arc::clone(&first.entry);
            remove_first(&mut node.right);
            true
        }
    };
    if removed {
        balance(link);
    }
    removed
}

fn remove_first<K, V>(link: &mut Link<K, V>) {
    if link.as_ref().expect("minimum subtree").left.is_none() {
        let next = owned(link).right.take();
        *link = next;
    } else {
        remove_first(&mut owned(link).left);
        balance(link);
    }
}

fn balance<K, V>(link: &mut Link<K, V>) {
    let node = link.as_ref().expect("remaining node");
    match node::rotation(&node.left, &node.right) {
        Rotation::None => {}
        Rotation::Right => rotate_right(link),
        Rotation::LeftRight => {
            rotate_left(&mut owned(link).left);
            rotate_right(link);
        }
        Rotation::Left => rotate_left(link),
        Rotation::RightLeft => {
            rotate_right(&mut owned(link).right);
            rotate_left(link);
        }
    }
    node::refresh_owned(owned(link));
}

fn rotate_right<K, V>(link: &mut Link<K, V>) {
    let mut root = link.take();
    let mut next = owned(&mut root).left.take();
    owned(&mut root).left = owned(&mut next).right.take();
    node::refresh_owned(owned(&mut root));
    owned(&mut next).right = root;
    node::refresh_owned(owned(&mut next));
    *link = next;
}

fn rotate_left<K, V>(link: &mut Link<K, V>) {
    let mut root = link.take();
    let mut next = owned(&mut root).right.take();
    owned(&mut root).right = owned(&mut next).left.take();
    node::refresh_owned(owned(&mut root));
    owned(&mut next).left = root;
    node::refresh_owned(owned(&mut next));
    *link = next;
}

#[cfg(test)]
#[path = "unique_tests.rs"]
mod tests;
