//! Structural-sharing diff between two `CowMap` AVL trees.
//!
//! Skips `Arc::ptr_eq` subtrees in `O(height)` via shared-suffix stack
//! truncation (and identical roots in `O(1)`), visiting only non-shared path
//! nodes even after AVL rotations reshaped the upper spine. Counting uses an
//! inline fixed-capacity traversal stack with zero heap allocations.

use std::cmp::Ordering;
use std::sync::Arc;

use super::{CowMap, Node};

/// Inline stack capacity covering any valid AVL tree (`height < 92` for `2^64`
/// elements) without heap allocation.
const INLINE_STACK_CAP: usize = 96;

pub(crate) enum CowDiff<'a, K, V> {
    /// Key is present in `self` and either absent in `base` or mapped to a
    /// different `Arc<V>` in `base`.
    Set(&'a K, &'a V, Option<&'a V>),
    /// Key is absent in `self` and present in `base`.
    Removed(&'a K, &'a V),
}

/// Exact step accounting for [`CowMap::visit_diff_against_with_stats`],
/// mirrored by `PersistentNaming.CowMapDeltaSeal.cursorDiff`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct CowDiffStats {
    pub(crate) visited: usize,
    pub(crate) ptr_checks: usize,
    pub(crate) push_steps: usize,
}

struct NodeStack<'a, K, V> {
    buf: [Option<&'a Arc<Node<K, V>>>; INLINE_STACK_CAP],
    len: usize,
    overflow: Vec<&'a Arc<Node<K, V>>>,
}

impl<'a, K, V> NodeStack<'a, K, V> {
    fn new() -> Self {
        Self {
            buf: [None; INLINE_STACK_CAP],
            len: 0,
            overflow: Vec::new(),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.len + self.overflow.len()
    }

    #[inline]
    fn push(&mut self, node: &'a Arc<Node<K, V>>) {
        if self.len < INLINE_STACK_CAP {
            self.buf[self.len] = Some(node);
            self.len += 1;
        } else {
            self.overflow.push(node);
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<&'a Arc<Node<K, V>>> {
        if let Some(node) = self.overflow.pop() {
            return Some(node);
        }
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.buf[self.len].take()
    }

    #[inline]
    fn last(&self) -> Option<&'a Arc<Node<K, V>>> {
        if let Some(&node) = self.overflow.last() {
            return Some(node);
        }
        self.len.checked_sub(1).and_then(|i| self.buf[i])
    }

    #[inline]
    fn get(&self, idx: usize) -> &'a Arc<Node<K, V>> {
        if idx < INLINE_STACK_CAP {
            self.buf[idx].expect("in-bounds inline stack slot")
        } else {
            self.overflow[idx - INLINE_STACK_CAP]
        }
    }

    #[inline]
    fn truncate(&mut self, new_len: usize) {
        if new_len <= INLINE_STACK_CAP {
            self.overflow.clear();
            self.len = new_len;
        } else {
            self.overflow.truncate(new_len - INLINE_STACK_CAP);
        }
    }
}

fn push_left_arc<'a, K, V>(
    mut node: &'a Option<Arc<Node<K, V>>>,
    stack: &mut NodeStack<'a, K, V>,
    push_steps: &mut usize,
) {
    while let Some(n) = node {
        stack.push(n);
        *push_steps += 1;
        node = &n.left;
    }
}

fn opt_ptr_eq<K, V>(a: &Option<Arc<Node<K, V>>>, b: &Option<Arc<Node<K, V>>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => Arc::ptr_eq(x, y),
        _ => false,
    }
}

impl<K, V> CowMap<K, V> {
    pub(crate) const fn len(&self) -> usize {
        self.len
    }
}

impl<K: Ord, V: PartialEq> CowMap<K, V> {
    /// Visits structural differences from `base` to `self` in strictly ascending
    /// key order with zero heap allocations. Stops early and returns `(false, stats)`
    /// when `visit` returns `false`; otherwise returns `(true, stats)`.
    pub(crate) fn visit_diff_against_with_stats<'a>(
        &'a self,
        base: &'a Self,
        mut visit: impl FnMut(CowDiff<'a, K, V>) -> bool,
    ) -> (bool, CowDiffStats) {
        let mut stats = CowDiffStats {
            visited: 0,
            ptr_checks: 1,
            push_steps: 0,
        };
        if opt_ptr_eq(&self.root, &base.root) {
            return (true, stats);
        }
        let mut sa = NodeStack::new();
        let mut sb = NodeStack::new();
        push_left_arc(&self.root, &mut sa, &mut stats.push_steps);
        push_left_arc(&base.root, &mut sb, &mut stats.push_steps);
        loop {
            match (sa.last(), sb.last()) {
                (None, None) => return (true, stats),
                (Some(a), Some(b)) => {
                    stats.ptr_checks += 1;
                    if Arc::ptr_eq(a, b) {
                        let (mut ia, mut ib) = (sa.len() - 1, sb.len() - 1);
                        while ia > 0 && ib > 0 {
                            stats.ptr_checks += 1;
                            if Arc::ptr_eq(sa.get(ia - 1), sb.get(ib - 1)) {
                                ia -= 1;
                                ib -= 1;
                            } else {
                                break;
                            }
                        }
                        sa.truncate(ia);
                        sb.truncate(ib);
                        continue;
                    }
                    stats.visited += 1;
                    match a.key.cmp(&b.key) {
                        Ordering::Less => {
                            let n = sa.pop().unwrap();
                            if !visit(CowDiff::Set(&n.key, n.val.as_ref(), None)) {
                                return (false, stats);
                            }
                            push_left_arc(&n.right, &mut sa, &mut stats.push_steps);
                        }
                        Ordering::Greater => {
                            let n = sb.pop().unwrap();
                            if !visit(CowDiff::Removed(&n.key, n.val.as_ref())) {
                                return (false, stats);
                            }
                            push_left_arc(&n.right, &mut sb, &mut stats.push_steps);
                        }
                        Ordering::Equal => {
                            let na = sa.pop().unwrap();
                            let nb = sb.pop().unwrap();
                            if !Arc::ptr_eq(&na.val, &nb.val) && na.val != nb.val {
                                if !visit(CowDiff::Set(
                                    &na.key,
                                    na.val.as_ref(),
                                    Some(nb.val.as_ref()),
                                )) {
                                    return (false, stats);
                                }
                            }
                            stats.ptr_checks += 1;
                            if !opt_ptr_eq(&na.right, &nb.right) {
                                push_left_arc(&na.right, &mut sa, &mut stats.push_steps);
                                push_left_arc(&nb.right, &mut sb, &mut stats.push_steps);
                            }
                        }
                    }
                }
                (Some(_), None) => {
                    let n = sa.pop().unwrap();
                    stats.visited += 1;
                    if !visit(CowDiff::Set(&n.key, n.val.as_ref(), None)) {
                        return (false, stats);
                    }
                    push_left_arc(&n.right, &mut sa, &mut stats.push_steps);
                }
                (None, Some(_)) => {
                    let n = sb.pop().unwrap();
                    stats.visited += 1;
                    if !visit(CowDiff::Removed(&n.key, n.val.as_ref())) {
                        return (false, stats);
                    }
                    push_left_arc(&n.right, &mut sb, &mut stats.push_steps);
                }
            }
        }
    }

    /// Computes the structural difference from `base` to `self`, returning
    /// `None` early if more than `limit` differing keys are encountered.
    pub(crate) fn diff_against_bounded_with_stats<'a>(
        &'a self,
        base: &'a Self,
        limit: usize,
    ) -> Option<(Vec<CowDiff<'a, K, V>>, CowDiffStats)> {
        if self.len.abs_diff(base.len) > limit {
            return None;
        }
        let mut out = Vec::new();
        let (completed, stats) = self.visit_diff_against_with_stats(base, |diff| {
            if out.len() >= limit {
                return false;
            }
            out.push(diff);
            true
        });
        completed.then_some((out, stats))
    }

    /// Computes the structural difference from `base` to `self`, returning
    /// `None` early if more than `limit` differing keys are encountered.
    ///
    /// Returns `(diffs, visited_nodes)` when `Some`, where `diffs` is strictly
    /// ascending by key and applying `diffs` to `base` reconstructs `self`.
    pub(crate) fn diff_against_bounded<'a>(
        &'a self,
        base: &'a Self,
        limit: usize,
    ) -> Option<(Vec<CowDiff<'a, K, V>>, usize)> {
        self.diff_against_bounded_with_stats(base, limit)
            .map(|(diffs, stats)| (diffs, stats.visited))
    }
}
