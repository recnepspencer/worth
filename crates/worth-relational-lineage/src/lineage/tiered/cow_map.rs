//! Structural copy-on-write persistent ordered map backed by an `Arc`-node
//! AVL tree.
//!
//! Cloning (`fork`) is `O(1)` (one root `Arc` refcount bump). Point lookups
//! and misses are strictly `O(log N)` with zero heap allocations regardless of
//! how many fork/write cycles occurred. Mutations copy only the `O(log N)`
//! nodes along the single modified path when shared, and update in place when
//! uniquely owned.

use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    val: Arc<V>,
    height: u8,
    left: Option<Arc<Node<K, V>>>,
    right: Option<Arc<Node<K, V>>>,
}

impl<K: Clone, V> Clone for Node<K, V> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            val: Arc::clone(&self.val),
            height: self.height,
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}

impl<K, V> Node<K, V> {
    fn leaf(key: K, val: Arc<V>) -> Arc<Self> {
        Arc::new(Self {
            key,
            val,
            height: 1,
            left: None,
            right: None,
        })
    }

    fn update_height(&mut self) {
        self.height = 1 + height(&self.left).max(height(&self.right));
    }

    fn balance_factor(&self) -> i32 {
        i32::from(height(&self.left)) - i32::from(height(&self.right))
    }
}

#[inline]
fn height<K, V>(node: &Option<Arc<Node<K, V>>>) -> u8 {
    node.as_ref().map_or(0, |n| n.height)
}

fn rotate_right<K: Clone, V>(mut y: Arc<Node<K, V>>) -> Arc<Node<K, V>> {
    let mut x = Arc::make_mut(&mut y)
        .left
        .take()
        .expect("rotate_right requires left child");
    let t2 = Arc::make_mut(&mut x).right.take();
    {
        let y_mut = Arc::make_mut(&mut y);
        y_mut.left = t2;
        y_mut.update_height();
    }
    {
        let x_mut = Arc::make_mut(&mut x);
        x_mut.right = Some(y);
        x_mut.update_height();
    }
    x
}

fn rotate_left<K: Clone, V>(mut x: Arc<Node<K, V>>) -> Arc<Node<K, V>> {
    let mut y = Arc::make_mut(&mut x)
        .right
        .take()
        .expect("rotate_left requires right child");
    let t2 = Arc::make_mut(&mut y).left.take();
    {
        let x_mut = Arc::make_mut(&mut x);
        x_mut.right = t2;
        x_mut.update_height();
    }
    {
        let y_mut = Arc::make_mut(&mut y);
        y_mut.left = Some(x);
        y_mut.update_height();
    }
    y
}

fn rebalance<K: Clone, V>(mut node: Arc<Node<K, V>>) -> Arc<Node<K, V>> {
    Arc::make_mut(&mut node).update_height();
    let bf = node.balance_factor();
    if bf > 1 {
        let left_bf = node.left.as_ref().map_or(0, |l| l.balance_factor());
        if left_bf < 0 {
            let left = Arc::make_mut(&mut node).left.take().unwrap();
            Arc::make_mut(&mut node).left = Some(rotate_left(left));
        }
        return rotate_right(node);
    }
    if bf < -1 {
        let right_bf = node.right.as_ref().map_or(0, |r| r.balance_factor());
        if right_bf > 0 {
            let right = Arc::make_mut(&mut node).right.take().unwrap();
            Arc::make_mut(&mut node).right = Some(rotate_right(right));
        }
        return rotate_left(node);
    }
    node
}

fn insert_node<K: Ord + Clone, V>(
    slot: Option<Arc<Node<K, V>>>,
    key: K,
    val: Arc<V>,
) -> (Arc<Node<K, V>>, bool) {
    let Some(mut node) = slot else {
        return (Node::leaf(key, val), true);
    };
    match key.cmp(&node.key) {
        Ordering::Less => {
            let left = Arc::make_mut(&mut node).left.take();
            let (new_left, inserted) = insert_node(left, key, val);
            Arc::make_mut(&mut node).left = Some(new_left);
            (rebalance(node), inserted)
        }
        Ordering::Greater => {
            let right = Arc::make_mut(&mut node).right.take();
            let (new_right, inserted) = insert_node(right, key, val);
            Arc::make_mut(&mut node).right = Some(new_right);
            (rebalance(node), inserted)
        }
        Ordering::Equal => {
            Arc::make_mut(&mut node).val = val;
            (node, false)
        }
    }
}

fn take_min<K: Clone, V>(mut node: Arc<Node<K, V>>) -> (K, Arc<V>, Option<Arc<Node<K, V>>>) {
    if node.left.is_none() {
        let right = Arc::make_mut(&mut node).right.take();
        match Arc::try_unwrap(node) {
            Ok(owned) => (owned.key, owned.val, right),
            Err(shared) => (shared.key.clone(), Arc::clone(&shared.val), right),
        }
    } else {
        let left = Arc::make_mut(&mut node).left.take().unwrap();
        let (k, v, new_left) = take_min(left);
        Arc::make_mut(&mut node).left = new_left;
        (k, v, Some(rebalance(node)))
    }
}

fn remove_node<K: Ord + Clone, V>(
    slot: Option<Arc<Node<K, V>>>,
    key: &K,
) -> (Option<Arc<Node<K, V>>>, bool) {
    let Some(mut node) = slot else {
        return (None, false);
    };
    match key.cmp(&node.key) {
        Ordering::Less => {
            let left = Arc::make_mut(&mut node).left.take();
            let (new_left, removed) = remove_node(left, key);
            Arc::make_mut(&mut node).left = new_left;
            if removed {
                (Some(rebalance(node)), true)
            } else {
                (Some(node), false)
            }
        }
        Ordering::Greater => {
            let right = Arc::make_mut(&mut node).right.take();
            let (new_right, removed) = remove_node(right, key);
            Arc::make_mut(&mut node).right = new_right;
            if removed {
                (Some(rebalance(node)), true)
            } else {
                (Some(node), false)
            }
        }
        Ordering::Equal => {
            let left = Arc::make_mut(&mut node).left.take();
            let right = Arc::make_mut(&mut node).right.take();
            match (left, right) {
                (None, r) => (r, true),
                (l, None) => (l, true),
                (Some(l), Some(r)) => {
                    let (min_k, min_v, new_r) = take_min(r);
                    let node_mut = Arc::make_mut(&mut node);
                    node_mut.key = min_k;
                    node_mut.val = min_v;
                    node_mut.left = Some(l);
                    node_mut.right = new_r;
                    (Some(rebalance(node)), true)
                }
            }
        }
    }
}

/// Persistent copy-on-write ordered map (`O(1)` clone, `O(log N)` lookup/update).
#[derive(Debug)]
pub(crate) struct CowMap<K, V> {
    root: Option<Arc<Node<K, V>>>,
    len: usize,
}

impl<K, V> Default for CowMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Clone for CowMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            len: self.len,
        }
    }
}

impl<K: Ord + PartialEq, V: PartialEq> PartialEq for CowMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        match (&self.root, &other.root) {
            (Some(a), Some(b)) if Arc::ptr_eq(a, b) => true,
            _ => self.iter().eq(other.iter()),
        }
    }
}

impl<K: Ord + Eq, V: Eq> Eq for CowMap<K, V> {}

impl<K, V> CowMap<K, V> {
    pub(crate) const fn new() -> Self {
        Self { root: None, len: 0 }
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }

    pub(crate) fn iter(&self) -> CowMapIter<'_, K, V> {
        let mut stack = Vec::new();
        push_left(&self.root, &mut stack);
        CowMapIter { stack }
    }
}

impl<K: Ord, V> CowMap<K, V> {
    pub(crate) fn get_arc(&self, key: &K) -> Option<&Arc<V>> {
        self.get_arc_and_shared(key).map(|(val, _)| val)
    }

    /// Returns the stored value `Arc` and whether it (or any AVL ancestor node
    /// on the path from `root` to `key`) is shared with another `CowMap` clone.
    pub(crate) fn get_arc_and_shared(&self, key: &K) -> Option<(&Arc<V>, bool)> {
        let mut cur = self.root.as_ref()?;
        let mut shared = false;
        loop {
            shared |= Arc::strong_count(cur) > 1;
            match key.cmp(&cur.key) {
                Ordering::Less => cur = cur.left.as_ref()?,
                Ordering::Greater => cur = cur.right.as_ref()?,
                Ordering::Equal => {
                    shared |= Arc::strong_count(&cur.val) > 1;
                    return Some((&cur.val, shared));
                }
            }
        }
    }

    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        self.get_arc(key).map(Arc::as_ref)
    }

    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Entries with keys at or above `lower`, ascending.
    ///
    /// Complexity: O(log N) to seek, then O(1) amortized per entry.
    pub(crate) fn iter_from(&self, lower: &K) -> CowMapIter<'_, K, V> {
        let mut stack = Vec::new();
        let mut node = &self.root;
        while let Some(n) = node {
            if n.key >= *lower {
                stack.push(n.as_ref());
                node = &n.left;
            } else {
                node = &n.right;
            }
        }
        CowMapIter { stack }
    }
}

impl<K: Ord + Clone, V> CowMap<K, V> {
    pub(crate) fn insert(&mut self, key: K, val: V) {
        self.insert_arc(key, Arc::new(val));
    }

    pub(crate) fn insert_arc(&mut self, key: K, val: Arc<V>) {
        let root = self.root.take();
        let (new_root, inserted) = insert_node(root, key, val);
        self.root = Some(new_root);
        if inserted {
            self.len += 1;
        }
    }

    pub(crate) fn remove(&mut self, key: &K) {
        if !self.contains_key(key) {
            return;
        }
        let root = self.root.take();
        let (new_root, removed) = remove_node(root, key);
        self.root = new_root;
        if removed {
            self.len -= 1;
        }
    }

    pub(crate) fn get_mut_with(
        &mut self,
        key: &K,
        duplicate: impl FnOnce(&V) -> V,
    ) -> Option<&mut V> {
        if !self.contains_key(key) {
            return None;
        }
        let mut cur = self.root.as_mut()?;
        loop {
            match key.cmp(&cur.key) {
                Ordering::Less => cur = Arc::make_mut(cur).left.as_mut()?,
                Ordering::Greater => cur = Arc::make_mut(cur).right.as_mut()?,
                Ordering::Equal => {
                    let val = &mut Arc::make_mut(cur).val;
                    if Arc::strong_count(val) > 1 {
                        *val = Arc::new(duplicate(val.as_ref()));
                    }
                    return Arc::get_mut(val);
                }
            }
        }
    }
}

fn push_left<'a, K, V>(mut node: &'a Option<Arc<Node<K, V>>>, stack: &mut Vec<&'a Node<K, V>>) {
    while let Some(n) = node {
        stack.push(n);
        node = &n.left;
    }
}

pub(crate) struct CowMapIter<'a, K, V> {
    stack: Vec<&'a Node<K, V>>,
}

impl<'a, K, V> Iterator for CowMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        push_left(&node.right, &mut self.stack);
        Some((&node.key, node.val.as_ref()))
    }
}
