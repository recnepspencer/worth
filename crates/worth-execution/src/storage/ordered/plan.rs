//! Bounded, borrowed AVL plans. Only the reachable replacement cells are allocated.
use super::{
    node::{self, Entry, Link, Node},
    ExecutionOrderedMapDenial as Denial,
};
use std::sync::Arc;
mod publish;
mod structure;

pub(super) enum PlannedLink<'tree, K, V> {
    Empty,
    Existing(&'tree Arc<Node<K, V>>),
    New(usize),
}
impl<K, V> Copy for PlannedLink<'_, K, V> {}
impl<K, V> Clone for PlannedLink<'_, K, V> {
    fn clone(&self) -> Self {
        *self
    }
}
struct PlannedNode<'tree, K, V> {
    entry: Option<&'tree Arc<Entry<K, V>>>,
    left: PlannedLink<'tree, K, V>,
    right: PlannedLink<'tree, K, V>,
    height: u8,
}
impl<K, V> Copy for PlannedNode<'_, K, V> {}
impl<K, V> Clone for PlannedNode<'_, K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

pub(super) struct Plan<'slots, 'tree, K, V> {
    slots: &'slots mut [Option<PlannedNode<'tree, K, V>>],
    used: usize,
}
impl<'tree, K, V> PlannedLink<'tree, K, V> {
    fn existing(link: &'tree Link<K, V>) -> Self {
        link.as_ref().map_or(Self::Empty, Self::Existing)
    }
}

pub(super) fn with_plan<'tree, K, V, R>(
    root: &'tree Link<K, V>,
    operation: impl FnOnce(&mut Plan<'_, 'tree, K, V>) -> Result<R, Denial>,
) -> Result<R, Denial> {
    // Keep larger inline planning frames outside the ordinary edit's frame.
    if node::height(root) > 32 {
        return with_large_plan(operation);
    }
    if node::height(root) > 16 {
        return with_medium_plan(operation);
    }
    let mut slots = [None; 3 * (16 + 1)];
    operation(&mut Plan {
        slots: &mut slots,
        used: 0,
    })
}
#[inline(never)]
fn with_medium_plan<'tree, K: 'tree, V: 'tree, R>(
    operation: impl FnOnce(&mut Plan<'_, 'tree, K, V>) -> Result<R, Denial>,
) -> Result<R, Denial> {
    let mut slots = [None; 3 * (32 + 1)];
    operation(&mut Plan {
        slots: &mut slots,
        used: 0,
    })
}
#[inline(never)]
fn with_large_plan<'tree, K: 'tree, V: 'tree, R>(
    operation: impl FnOnce(&mut Plan<'_, 'tree, K, V>) -> Result<R, Denial>,
) -> Result<R, Denial> {
    // One balancing step plans <=3 cells; an edit is bounded by the original
    // AVL height, including its successor descent. Height is stored as u8.
    let mut slots = [None; 3 * (u8::MAX as usize + 1)];
    operation(&mut Plan {
        slots: &mut slots,
        used: 0,
    })
}

impl<'tree, K: Ord, V> Plan<'_, 'tree, K, V> {
    pub(super) fn insert(
        &mut self,
        root: &'tree Link<K, V>,
        key: &K,
    ) -> Result<PlannedLink<'tree, K, V>, Denial> {
        self.insert_at(PlannedLink::existing(root), key)
    }
    fn insert_at(
        &mut self,
        root: PlannedLink<'tree, K, V>,
        key: &K,
    ) -> Result<PlannedLink<'tree, K, V>, Denial> {
        let Some(old) = self.view(root) else {
            return self.build(None, PlannedLink::Empty, PlannedLink::Empty);
        };
        match key.cmp(&old.entry.expect("only existing entries are searched").key) {
            std::cmp::Ordering::Equal => self.build(None, old.left, old.right),
            std::cmp::Ordering::Less => {
                let left = self.insert_at(old.left, key)?;
                self.balanced(old.entry, left, old.right)
            }
            std::cmp::Ordering::Greater => {
                let right = self.insert_at(old.right, key)?;
                self.balanced(old.entry, old.left, right)
            }
        }
    }
    pub(super) fn remove(
        &mut self,
        root: &'tree Link<K, V>,
        key: &K,
    ) -> Result<PlannedLink<'tree, K, V>, Denial> {
        self.remove_at(PlannedLink::existing(root), key)
    }
    fn remove_at(
        &mut self,
        root: PlannedLink<'tree, K, V>,
        key: &K,
    ) -> Result<PlannedLink<'tree, K, V>, Denial> {
        let Some(old) = self.view(root) else {
            return Ok(PlannedLink::Empty);
        };
        match key.cmp(&old.entry.expect("removal has no inserted entry").key) {
            std::cmp::Ordering::Less => {
                let left = self.remove_at(old.left, key)?;
                self.balanced(old.entry, left, old.right)
            }
            std::cmp::Ordering::Greater => {
                let right = self.remove_at(old.right, key)?;
                self.balanced(old.entry, old.left, right)
            }
            std::cmp::Ordering::Equal => match (old.left, old.right) {
                (PlannedLink::Empty, _) => Ok(old.right),
                (_, PlannedLink::Empty) => Ok(old.left),
                _ => {
                    let mut first = self.view(old.right).expect("right child");
                    while let Some(left) = self.view(first.left) {
                        first = left;
                    }
                    let entry = first.entry.expect("existing successor");
                    let right = self.remove_at(old.right, &entry.key)?;
                    self.balanced(Some(entry), old.left, right)
                }
            },
        }
    }
}
impl<'tree, K, V> Plan<'_, 'tree, K, V> {
    fn view(&self, link: PlannedLink<'tree, K, V>) -> Option<PlannedNode<'tree, K, V>> {
        match link {
            PlannedLink::Empty => None,
            PlannedLink::New(index) => self.slots[index],
            PlannedLink::Existing(node) => Some(PlannedNode {
                entry: Some(&node.entry),
                left: PlannedLink::existing(&node.left),
                right: PlannedLink::existing(&node.right),
                height: node.height,
            }),
        }
    }
    fn height(&self, link: PlannedLink<'tree, K, V>) -> u8 {
        self.view(link).map_or(0, |n| n.height)
    }
    fn build(
        &mut self,
        entry: Option<&'tree Arc<Entry<K, V>>>,
        left: PlannedLink<'tree, K, V>,
        right: PlannedLink<'tree, K, V>,
    ) -> Result<PlannedLink<'tree, K, V>, Denial> {
        let height = self
            .height(left)
            .max(self.height(right))
            .checked_add(1)
            .ok_or(Denial::CapacityOverflow)?;
        let slot = self
            .slots
            .get_mut(self.used)
            .ok_or(Denial::CapacityOverflow)?;
        *slot = Some(PlannedNode {
            entry,
            left,
            right,
            height,
        });
        self.used += 1;
        Ok(PlannedLink::New(self.used - 1))
    }
}
