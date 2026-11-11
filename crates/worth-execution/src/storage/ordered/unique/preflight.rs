//! Borrow-only validation of every cell an edit will mutate, before effects.
use super::ExecutionOrderedMapUniqueDenial as Denial;
use crate::storage::ordered::node::{self, height, Link, Node};
use std::{cmp::Ordering, sync::Arc};

pub(super) fn set<K: Ord, V>(root: &mut Link<K, V>, key: &K) -> Result<bool, Denial> {
    let Some(root) = root else { return Ok(false) };
    let node = Arc::get_mut(root).ok_or(Denial::SharedStorage)?;
    match key.cmp(&node.entry.key) {
        Ordering::Less => set(&mut node.left, key),
        Ordering::Greater => set(&mut node.right, key),
        Ordering::Equal => {
            Arc::get_mut(&mut node.entry).ok_or(Denial::SharedStorage)?;
            Ok(true)
        }
    }
}

pub(super) fn insert<K: Ord, V>(
    root: &mut Link<K, V>,
    key: &K,
    bytes: Option<u64>,
) -> Result<(), Denial> {
    let Some(root) = root else { return Ok(()) };
    let node = Arc::get_mut(root).ok_or(Denial::SharedStorage)?;
    node.count.checked_add(1).ok_or(Denial::CapacityOverflow)?;
    if let (Some(old), Some(added)) = (node.bytes, bytes) {
        old.checked_add(added).ok_or(Denial::CapacityOverflow)?;
    }
    match key.cmp(&node.entry.key) {
        Ordering::Less => insert(&mut node.left, key, bytes),
        Ordering::Greater => insert(&mut node.right, key, bytes),
        Ordering::Equal => Err(Denial::Occupied),
    }
    // Insertion rotations touch only nodes on this admitted search path.
}

pub(super) fn remove<K: Ord, V>(root: &mut Link<K, V>, key: &K) -> Result<Option<u8>, Denial> {
    let Some(root) = root else { return Ok(None) };
    let node = Arc::get_mut(root).ok_or(Denial::SharedStorage)?;
    let (left, right) = match key.cmp(&node.entry.key) {
        Ordering::Less => {
            let Some(left) = remove(&mut node.left, key)? else {
                return Ok(None);
            };
            (left, height(&node.right))
        }
        Ordering::Greater => {
            let Some(right) = remove(&mut node.right, key)? else {
                return Ok(None);
            };
            (height(&node.left), right)
        }
        Ordering::Equal => {
            let entry = Arc::get_mut(&mut node.entry).ok_or(Denial::SharedStorage)?;
            node::detached_bytes::<K, V>(entry).map_err(|_| Denial::CapacityOverflow)?;
            if node.left.is_none() || node.right.is_none() {
                return Ok(Some(height(&node.left).max(height(&node.right))));
            }
            (height(&node.left), remove_first(&mut node.right)?)
        }
    };
    rebalanced_height(node, left, right).map(Some)
}

fn remove_first<K, V>(root: &mut Link<K, V>) -> Result<u8, Denial> {
    let node = Arc::get_mut(root.as_mut().expect("nonempty successor subtree"))
        .ok_or(Denial::SharedStorage)?;
    if node.left.is_none() {
        return Ok(height(&node.right));
    }
    let left = remove_first(&mut node.left)?;
    let right = height(&node.right);
    rebalanced_height(node, left, right)
}

fn rebalanced_height<K, V>(node: &mut Node<K, V>, left: u8, right: u8) -> Result<u8, Denial> {
    // Deletion can make only its unchanged sibling heavy. Validate that exact
    // child (and the inner pivot for a double rotation) before moving links.
    if left > right + 1 {
        let child = Arc::get_mut(node.left.as_mut().expect("left-heavy subtree"))
            .ok_or(Denial::SharedStorage)?;
        let outer = height(&child.left);
        let inner = height(&child.right);
        if outer >= inner {
            return Ok(outer.max(inner.max(right) + 1) + 1);
        }
        let pivot = Arc::get_mut(child.right.as_mut().expect("inner rotation pivot"))
            .ok_or(Denial::SharedStorage)?;
        Ok((outer.max(height(&pivot.left)) + 1).max(height(&pivot.right).max(right) + 1) + 1)
    } else if right > left + 1 {
        let child = Arc::get_mut(node.right.as_mut().expect("right-heavy subtree"))
            .ok_or(Denial::SharedStorage)?;
        let outer = height(&child.right);
        let inner = height(&child.left);
        if outer >= inner {
            return Ok(outer.max(inner.max(left) + 1) + 1);
        }
        let pivot = Arc::get_mut(child.left.as_mut().expect("inner rotation pivot"))
            .ok_or(Denial::SharedStorage)?;
        Ok((left.max(height(&pivot.left)) + 1).max(height(&pivot.right).max(outer) + 1) + 1)
    } else {
        Ok(left.max(right) + 1)
    }
}
