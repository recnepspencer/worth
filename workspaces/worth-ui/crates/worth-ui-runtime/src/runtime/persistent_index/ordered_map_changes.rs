use std::cmp::Ordering;
use std::rc::Rc;

use super::ordered_map::{Node, UiPersistentOrdMap};

/// Comparison work includes cursor alignment and shared-subtree checks.
/// Shared versions visit changed search paths; unrelated versions may visit
/// their entire contents. This comparison does not retain a change journal.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UiPersistentMapComparisonWork {
    cursor_steps: usize,
    shared_subtrees_skipped: usize,
}

impl UiPersistentMapComparisonWork {
    pub(crate) const fn cursor_steps(self) -> usize {
        self.cursor_steps
    }

    #[cfg(test)]
    pub(crate) const fn shared_subtrees_skipped(self) -> usize {
        self.shared_subtrees_skipped
    }
}

impl<K: Ord + Clone, V: Eq> UiPersistentOrdMap<K, V> {
    pub(crate) fn changed_keys_with_work(
        &self,
        predecessor: &Self,
    ) -> (Vec<K>, UiPersistentMapComparisonWork) {
        let mut current = Cursor::new(self.root.as_ref());
        let mut previous = Cursor::new(predecessor.root.as_ref());
        let mut changed = Vec::new();
        let mut work = UiPersistentMapComparisonWork::default();
        while !current.stack.is_empty() || !previous.stack.is_empty() {
            #[cfg(test)]
            super::test_observation::observe_comparison_step();
            work.cursor_steps += 1;
            match (current.stack.last(), previous.stack.last()) {
                (Some(Part::Tree(a)), Some(Part::Tree(b))) => {
                    if Rc::ptr_eq(a, b) {
                        current.stack.pop();
                        previous.stack.pop();
                        work.shared_subtrees_skipped += 1;
                    } else {
                        // Expand the later root first so an earlier shared
                        // subtree stays intact across an AVL rotation.
                        match a.key.cmp(&b.key) {
                            Ordering::Less => previous.expand(),
                            Ordering::Greater => current.expand(),
                            Ordering::Equal => {
                                current.expand();
                                previous.expand();
                            }
                        }
                    }
                }
                (Some(Part::Tree(_)), _) => current.expand(),
                (_, Some(Part::Tree(_))) => previous.expand(),
                (Some(Part::Entry(a)), Some(Part::Entry(b))) => match a.key.cmp(&b.key) {
                    Ordering::Less => changed.push(current.take_key()),
                    Ordering::Greater => changed.push(previous.take_key()),
                    Ordering::Equal => {
                        if a.value != b.value {
                            changed.push(a.key.clone());
                        }
                        current.stack.pop();
                        previous.stack.pop();
                    }
                },
                (Some(Part::Entry(_)), None) => changed.push(current.take_key()),
                (None, Some(Part::Entry(_))) => changed.push(previous.take_key()),
                (None, None) => unreachable!("comparison has remaining work"),
            }
        }
        (changed, work)
    }
}

enum Part<'a, K, V> {
    Tree(&'a Rc<Node<K, V>>),
    Entry(&'a Node<K, V>),
}

struct Cursor<'a, K, V> {
    stack: Vec<Part<'a, K, V>>,
}

impl<'a, K: Clone, V> Cursor<'a, K, V> {
    fn new(root: Option<&'a Rc<Node<K, V>>>) -> Self {
        Self {
            stack: root.into_iter().map(Part::Tree).collect(),
        }
    }

    fn expand(&mut self) {
        let Some(Part::Tree(node)) = self.stack.pop() else {
            unreachable!("only a tree cursor can expand");
        };
        if let Some(right) = node.right.as_ref() {
            self.stack.push(Part::Tree(right));
        }
        self.stack.push(Part::Entry(node));
        if let Some(left) = node.left.as_ref() {
            self.stack.push(Part::Tree(left));
        }
    }

    fn take_key(&mut self) -> K {
        let Some(Part::Entry(node)) = self.stack.pop() else {
            unreachable!("only an entry cursor can emit a key");
        };
        node.key.clone()
    }
}

#[cfg(test)]
#[path = "ordered_map_changes_tests.rs"]
mod tests;
