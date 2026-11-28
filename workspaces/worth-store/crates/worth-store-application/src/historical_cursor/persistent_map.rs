//! Persistent `Arc`-shared big-endian Patricia trie over 128-bit record keys
//! (Okasaki & Gill, 1998) for single-pass topological DAG replay.
//!
//! Structural properties:
//! * `clone` is `O(1)` (`Arc::clone` of root).
//! * `get` and `insert` are `O(log N)` (at most 128 levels) while preserving
//!   structural sharing across all untouched subtrees.

use std::sync::Arc;

enum Node<V> {
    Leaf {
        key: u128,
        value: Arc<V>,
    },
    Branch {
        prefix: u128,
        bit: u32,
        left: Arc<Node<V>>,
        right: Arc<Node<V>>,
    },
}

/// Immutable, `Arc`-shared ordered map from `u128` to `Arc<V>`.
pub(super) struct PersistentU128Map<V> {
    root: Option<Arc<Node<V>>>,
}

impl<V> Clone for PersistentU128Map<V> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.as_ref().map(Arc::clone),
        }
    }
}

impl<V> Default for PersistentU128Map<V> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<V> PersistentU128Map<V> {
    pub(super) fn get(&self, key: u128, steps: &mut usize) -> Option<&Arc<V>> {
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            *steps += 1;
            match node {
                Node::Leaf { key: k, value } => {
                    return (*k == key).then_some(value);
                }
                Node::Branch {
                    prefix,
                    bit,
                    left,
                    right,
                } => {
                    if !match_prefix(key, *prefix, *bit) {
                        return None;
                    }
                    current = Some(if zero_bit(key, *bit) {
                        left.as_ref()
                    } else {
                        right.as_ref()
                    });
                }
            }
        }
        None
    }

    pub(super) fn insert(&self, key: u128, value: Arc<V>, steps: &mut usize) -> Self {
        let root = match self.root.as_ref() {
            None => {
                *steps += 1;
                Arc::new(Node::Leaf { key, value })
            }
            Some(node) => insert_node(node, key, value, steps),
        };
        Self { root: Some(root) }
    }
}

#[inline]
const fn zero_bit(key: u128, bit: u32) -> bool {
    (key & (1u128 << bit)) == 0
}

#[inline]
const fn highest_diff_bit(a: u128, b: u128) -> u32 {
    127 - (a ^ b).leading_zeros()
}

#[inline]
const fn mask_prefix(key: u128, bit: u32) -> u128 {
    if bit >= 127 {
        0
    } else {
        key & (!0u128 << (bit + 1))
    }
}

#[inline]
const fn match_prefix(key: u128, prefix: u128, bit: u32) -> bool {
    mask_prefix(key, bit) == prefix
}

fn join<V>(p0: u128, t0: Arc<Node<V>>, p1: u128, t1: Arc<Node<V>>) -> Arc<Node<V>> {
    let bit = highest_diff_bit(p0, p1);
    let prefix = mask_prefix(p0, bit);
    if zero_bit(p0, bit) {
        Arc::new(Node::Branch {
            prefix,
            bit,
            left: t0,
            right: t1,
        })
    } else {
        Arc::new(Node::Branch {
            prefix,
            bit,
            left: t1,
            right: t0,
        })
    }
}

fn rebuild_branch<V>(
    orig: &Arc<Node<V>>,
    prefix: u128,
    bit: u32,
    orig_left: &Arc<Node<V>>,
    orig_right: &Arc<Node<V>>,
    new_left: Arc<Node<V>>,
    new_right: Arc<Node<V>>,
) -> Arc<Node<V>> {
    if Arc::ptr_eq(orig_left, &new_left) && Arc::ptr_eq(orig_right, &new_right) {
        Arc::clone(orig)
    } else {
        Arc::new(Node::Branch {
            prefix,
            bit,
            left: new_left,
            right: new_right,
        })
    }
}

fn insert_node<V>(
    node: &Arc<Node<V>>,
    key: u128,
    value: Arc<V>,
    steps: &mut usize,
) -> Arc<Node<V>> {
    *steps += 1;
    match node.as_ref() {
        Node::Leaf { key: k, value: v } => {
            if *k == key {
                if Arc::ptr_eq(v, &value) {
                    Arc::clone(node)
                } else {
                    Arc::new(Node::Leaf { key, value })
                }
            } else {
                join(
                    key,
                    Arc::new(Node::Leaf { key, value }),
                    *k,
                    Arc::clone(node),
                )
            }
        }
        Node::Branch {
            prefix,
            bit,
            left,
            right,
        } => {
            if !match_prefix(key, *prefix, *bit) {
                join(
                    key,
                    Arc::new(Node::Leaf { key, value }),
                    *prefix,
                    Arc::clone(node),
                )
            } else if zero_bit(key, *bit) {
                let nl = insert_node(left, key, value, steps);
                rebuild_branch(node, *prefix, *bit, left, right, nl, Arc::clone(right))
            } else {
                let nr = insert_node(right, key, value, steps);
                rebuild_branch(node, *prefix, *bit, left, right, Arc::clone(left), nr)
            }
        }
    }
}
