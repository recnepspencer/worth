//! Move links of preflighted unique cells. No allocation, Clone or fallback.
use crate::storage::ordered::node::{height, refresh, Link, Node};
use std::{cmp::Ordering, sync::Arc};

fn unique<K, V>(node: &mut Arc<Node<K, V>>) -> &mut Node<K, V> {
    Arc::get_mut(node).expect("every mutated node passed uniqueness preflight")
}

pub(super) fn set<K: Ord, V>(root: &mut Link<K, V>, key: &K, value: V) -> V {
    let node = unique(root.as_mut().expect("preflight found the existing key"));
    match key.cmp(&node.entry.key) {
        Ordering::Less => set(&mut node.left, key, value),
        Ordering::Greater => set(&mut node.right, key, value),
        Ordering::Equal => std::mem::replace(
            &mut Arc::get_mut(&mut node.entry)
                .expect("the updated entry passed uniqueness preflight")
                .value,
            value,
        ),
    }
}

pub(super) fn insert<K: Ord, V>(root: &mut Link<K, V>, leaf: Arc<Node<K, V>>) {
    let Some(mut original) = root.take() else {
        *root = Some(leaf);
        return;
    };
    let node = unique(&mut original);
    match leaf.entry.key.cmp(&node.entry.key) {
        Ordering::Less => insert(&mut node.left, leaf),
        Ordering::Greater => insert(&mut node.right, leaf),
        Ordering::Equal => unreachable!("insertion preflight requires a vacant key"),
    }
    *root = Some(balanced(original));
}

pub(super) fn remove<K: Ord, V>(root: &mut Link<K, V>, key: &K) -> Arc<Node<K, V>> {
    let mut original = root.take().expect("preflight found the existing key");
    let node = unique(&mut original);
    let removed = match key.cmp(&node.entry.key) {
        Ordering::Less => remove(&mut node.left, key),
        Ordering::Greater => remove(&mut node.right, key),
        Ordering::Equal => {
            if node.left.is_none() || node.right.is_none() {
                *root = node.left.take().or_else(|| node.right.take());
                refresh(node);
                return original;
            }
            let mut successor = remove_first(&mut node.right);
            let replacement = unique(&mut successor);
            replacement.left = node.left.take();
            replacement.right = node.right.take();
            // Preserve conservative accounting knowledge for the replacement
            // subtree, then recompute its actual surviving cells during balance.
            replacement.bytes = node.bytes;
            refresh(node);
            *root = Some(balanced(successor));
            return original;
        }
    };
    *root = Some(balanced(original));
    removed
}

fn remove_first<K, V>(root: &mut Link<K, V>) -> Arc<Node<K, V>> {
    let mut original = root.take().expect("nonempty successor subtree");
    let node = unique(&mut original);
    if node.left.is_none() {
        *root = node.right.take();
        return original;
    }
    let removed = remove_first(&mut node.left);
    *root = Some(balanced(original));
    removed
}

fn balanced<K, V>(mut root: Arc<Node<K, V>>) -> Arc<Node<K, V>> {
    let node = unique(&mut root);
    if height(&node.left) > height(&node.right) + 1 {
        let child = node.left.as_ref().expect("left-heavy node");
        if height(&child.left) < height(&child.right) {
            node.left = Some(rotate_left(node.left.take().expect("left child")));
        }
        rotate_right(root)
    } else if height(&node.right) > height(&node.left) + 1 {
        let child = node.right.as_ref().expect("right-heavy node");
        if height(&child.right) < height(&child.left) {
            node.right = Some(rotate_right(node.right.take().expect("right child")));
        }
        rotate_left(root)
    } else {
        refresh(node);
        root
    }
}

fn rotate_right<K, V>(mut root: Arc<Node<K, V>>) -> Arc<Node<K, V>> {
    let node = unique(&mut root);
    let mut promoted = node.left.take().expect("right rotation has a left child");
    node.left = unique(&mut promoted).right.take();
    refresh(node);
    unique(&mut promoted).right = Some(root);
    refresh(unique(&mut promoted));
    promoted
}

fn rotate_left<K, V>(mut root: Arc<Node<K, V>>) -> Arc<Node<K, V>> {
    let node = unique(&mut root);
    let mut promoted = node.right.take().expect("left rotation has a right child");
    node.right = unique(&mut promoted).left.take();
    refresh(node);
    unique(&mut promoted).left = Some(root);
    refresh(unique(&mut promoted));
    promoted
}
