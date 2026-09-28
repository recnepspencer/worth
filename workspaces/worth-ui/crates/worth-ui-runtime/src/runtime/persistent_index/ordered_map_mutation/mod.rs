use std::cmp::Ordering;
use std::rc::Rc;

use super::mutation_work::UiPersistentIndexMutationWork;
use super::ordered_map::{height, node_len, Link, Node};

/// A path node's copy shares its value and both subtrees.
impl<K: Clone, V> Clone for Node<K, V> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            value: Rc::clone(&self.value),
            left: self.left.clone(),
            right: self.right.clone(),
            height: self.height,
            len: self.len,
        }
    }
}

pub(super) fn insert<K: Ord + Clone, V>(
    root: Link<K, V>,
    key: K,
    value: Rc<V>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    let Some(mut node) = root else {
        return leaf(key, value, work);
    };
    work.record_key_probe();
    match key.cmp(&node.key) {
        Ordering::Less => {
            let left = insert(take_left(&mut node), key, value, work);
            balance(with_left(node, Some(left), work), work)
        }
        Ordering::Greater => {
            let right = insert(take_right(&mut node), key, value, work);
            balance(with_right(node, Some(right), work), work)
        }
        Ordering::Equal => {
            let rebuilt = rebuild(&mut node, work);
            rebuilt.key = key;
            rebuilt.value = value;
            node
        }
    }
}

impl<K: Ord + Clone, V> super::UiPersistentOrdMap<K, V> {
    /// Applies `edit` to `key`'s value, or to the default when absent. The
    /// value moves out and back, so it is copied only while a fork shares it.
    pub(crate) fn edit_or_default(&mut self, key: K, edit: impl FnOnce(&mut V))
    where
        V: Clone + Default,
    {
        let mut value = self.take_with_work(&key).0.unwrap_or_default();
        edit(&mut value);
        self.insert(key, value);
    }

    /// Removes `key` and hands back its value, copying it only while a fork
    /// still shares it.
    pub(crate) fn take_with_work(&mut self, key: &K) -> (Option<V>, UiPersistentIndexMutationWork)
    where
        V: Clone,
    {
        let mut work = UiPersistentIndexMutationWork::default();
        let (root, removed) = remove(self.root.take(), key, &mut work);
        self.root = root;
        let value =
            removed.map(|value| Rc::try_unwrap(value).unwrap_or_else(|shared| (*shared).clone()));
        (value, work)
    }
}

/// Removes `key`, answering the removed value. A removed node with one child
/// is moved out whole; a two-child node's value is swapped out as its
/// successor takes its place. Either way an unshared value leaves with its
/// only owner.
pub(super) fn remove<K: Ord + Clone, V>(
    root: Link<K, V>,
    key: &K,
    work: &mut UiPersistentIndexMutationWork,
) -> (Link<K, V>, Option<Rc<V>>) {
    let Some(mut node) = root else {
        return (None, None);
    };
    work.record_key_probe();
    match key.cmp(&node.key) {
        Ordering::Less => {
            let (left, removed) = remove(take_left(&mut node), key, work);
            (Some(balance(with_left(node, left, work), work)), removed)
        }
        Ordering::Greater => {
            let (right, removed) = remove(take_right(&mut node), key, work);
            (Some(balance(with_right(node, right, work), work)), removed)
        }
        Ordering::Equal if node.left.is_some() && node.right.is_some() => {
            let right = take_right(&mut node).expect("a two-child node has a right subtree");
            let (successor_key, successor_value, next_right) = take_min(right, work);
            let rebuilt = rebuild(&mut node, work);
            rebuilt.key = successor_key;
            let removed = std::mem::replace(&mut rebuilt.value, successor_value);
            rebuilt.right = next_right;
            refresh(rebuilt);
            (Some(balance(node, work)), Some(removed))
        }
        Ordering::Equal => {
            let (child, removed) = detach(node);
            (child, Some(removed))
        }
    }
}

fn take_min<K: Ord + Clone, V>(
    mut node: Rc<Node<K, V>>,
    work: &mut UiPersistentIndexMutationWork,
) -> (K, Rc<V>, Link<K, V>) {
    let Some(left) = take_left(&mut node) else {
        return match Rc::try_unwrap(node) {
            Ok(owned) => (owned.key, owned.value, owned.right),
            Err(shared) => (
                shared.key.clone(),
                Rc::clone(&shared.value),
                shared.right.clone(),
            ),
        };
    };
    let (key, value, next_left) = take_min(left, work);
    let rebalanced = balance(with_left(node, next_left, work), work);
    (key, value, Some(rebalanced))
}

/// A removed node with at most one child: that child and the node's value,
/// moved out when nothing else holds the node.
fn detach<K, V>(node: Rc<Node<K, V>>) -> (Link<K, V>, Rc<V>) {
    match Rc::try_unwrap(node) {
        Ok(owned) => (owned.left.or(owned.right), owned.value),
        Err(shared) => (
            shared.left.clone().or_else(|| shared.right.clone()),
            Rc::clone(&shared.value),
        ),
    }
}

fn balance<K: Ord + Clone, V>(
    mut node: Rc<Node<K, V>>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    let skew = height(&node.left) as i32 - height(&node.right) as i32;
    if skew > 1 {
        let left = node
            .left
            .as_ref()
            .expect("left-heavy node has a left child");
        if height(&left.left) >= height(&left.right) {
            return rotate_right(node, work);
        }
        let left = take_left(&mut node).expect("left-heavy node has a left child");
        let rotated = rotate_left(left, work);
        return rotate_right(with_left(node, Some(rotated), work), work);
    }
    if skew < -1 {
        let right = node
            .right
            .as_ref()
            .expect("right-heavy node has a right child");
        if height(&right.right) >= height(&right.left) {
            return rotate_left(node, work);
        }
        let right = take_right(&mut node).expect("right-heavy node has a right child");
        let rotated = rotate_right(right, work);
        return rotate_left(with_right(node, Some(rotated), work), work);
    }
    node
}

fn rotate_left<K: Clone, V>(
    mut root: Rc<Node<K, V>>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    let mut pivot = take_right(&mut root).expect("left rotation requires right child");
    let inner = take_left(&mut pivot);
    let lowered = rebuild(&mut root, work);
    lowered.right = inner;
    refresh(lowered);
    let raised = rebuild(&mut pivot, work);
    raised.left = Some(root);
    refresh(raised);
    pivot
}

fn rotate_right<K: Clone, V>(
    mut root: Rc<Node<K, V>>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    let mut pivot = take_left(&mut root).expect("right rotation requires left child");
    let inner = take_right(&mut pivot);
    let lowered = rebuild(&mut root, work);
    lowered.left = inner;
    refresh(lowered);
    let raised = rebuild(&mut pivot, work);
    raised.right = Some(root);
    refresh(raised);
    pivot
}

fn with_left<K: Clone, V>(
    mut node: Rc<Node<K, V>>,
    left: Link<K, V>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    let rebuilt = rebuild(&mut node, work);
    rebuilt.left = left;
    refresh(rebuilt);
    node
}

fn with_right<K: Clone, V>(
    mut node: Rc<Node<K, V>>,
    right: Link<K, V>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    let rebuilt = rebuild(&mut node, work);
    rebuilt.right = right;
    refresh(rebuilt);
    node
}

/// Detaches the left subtree to descend into. An unshared node gives it up,
/// so the subtree stays unshared too; a shared node lends another reference,
/// and the descent copies the path beneath it.
fn take_left<K, V>(node: &mut Rc<Node<K, V>>) -> Link<K, V> {
    match Rc::get_mut(node) {
        Some(unshared) => unshared.left.take(),
        None => node.left.clone(),
    }
}

fn take_right<K, V>(node: &mut Rc<Node<K, V>>) -> Link<K, V> {
    match Rc::get_mut(node) {
        Some(unshared) => unshared.right.take(),
        None => node.right.clone(),
    }
}

/// One rebuilt path node: rewritten in place when nothing else holds it,
/// copied while a fork shares it, so a fork never observes the rewrite.
fn rebuild<'node, K: Clone, V>(
    node: &'node mut Rc<Node<K, V>>,
    work: &mut UiPersistentIndexMutationWork,
) -> &'node mut Node<K, V> {
    work.record_node_copy(Rc::get_mut(node).is_none());
    Rc::make_mut(node)
}

fn refresh<K, V>(node: &mut Node<K, V>) {
    node.height = 1 + height(&node.left).max(height(&node.right));
    node.len = 1 + node_len(&node.left) + node_len(&node.right);
}

fn leaf<K, V>(key: K, value: Rc<V>, work: &mut UiPersistentIndexMutationWork) -> Rc<Node<K, V>> {
    work.record_node_copy(true);
    Rc::new(Node {
        key,
        value,
        left: None,
        right: None,
        height: 1,
        len: 1,
    })
}

#[cfg(test)]
mod tests;
