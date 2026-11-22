//! Immutable aspect entries: path-copying AVL updates and allocation-free ordered reads.
//! Each `Arc<EntryNode>` exposes its physical allocation identity for exact deduplication.
mod allocation;

use crate::aspects::keys::AspectKey;
use crate::aspects::validation::ContractValidatedAspectValue;
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::sync::Arc;

type Link = Option<Arc<EntryNode>>;

struct EntryNode {
    value: ContractValidatedAspectValue,
    left: Link,
    right: Link,
    height: usize,
    len: usize,
    birth_seq: Option<u64>,
}

#[derive(Clone, Default)]
pub(in crate::aspects::state) struct AspectEntryTree {
    root: Link,
}

impl AspectEntryTree {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Consume a strictly sorted stream into a balanced AVL tree with zero path-copy churn.
    pub(super) fn from_sorted_values(
        mut values: impl ExactSizeIterator<Item = ContractValidatedAspectValue>,
    ) -> Self {
        let count = values.len();
        Self {
            root: sorted_node(&mut values, count),
        }
    }

    pub(super) fn len(&self) -> usize {
        length(&self.root)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub(super) fn contains_key(&self, key: &AspectKey) -> bool {
        self.get(key).is_some()
    }

    pub(super) fn get(&self, key: &AspectKey) -> Option<&ContractValidatedAspectValue> {
        lookup_node(&self.root, key).map(|node| &node.value)
    }

    #[cfg(test)]
    pub(in crate::aspects::state) fn shares_root(&self, other: &Self) -> bool {
        option_arc_ptr_eq(&self.root, &other.root)
    }

    pub(super) fn insert(
        &mut self,
        value: ContractValidatedAspectValue,
    ) -> Option<ContractValidatedAspectValue> {
        let (next, previous) = insert_node(&self.root, value);
        self.root = Some(next);
        previous
    }

    pub(super) fn remove(&mut self, key: &AspectKey) -> Option<ContractValidatedAspectValue> {
        let (next, removed) = remove_node(&self.root, key);
        if removed.is_some() {
            self.root = next;
        }
        removed
    }

    pub(super) fn iter(&self) -> Entries<'_> {
        let mut entries = Entries {
            pending: [None; STACK],
            depth: 0,
            remaining: self.len(),
        };
        entries.descend(self.root.as_deref());
        entries
    }
}

fn option_arc_ptr_eq(left: &Link, right: &Link) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        _ => false,
    }
}

fn lookup_node<'a>(root: &'a Link, key: &AspectKey) -> Option<&'a Arc<EntryNode>> {
    let mut current = root.as_ref();
    while let Some(node) = current {
        match key.cmp(node.value.key()) {
            std::cmp::Ordering::Less => current = node.left.as_ref(),
            std::cmp::Ordering::Equal => return Some(node),
            std::cmp::Ordering::Greater => current = node.right.as_ref(),
        }
    }
    None
}

fn sorted_node(
    values: &mut impl ExactSizeIterator<Item = ContractValidatedAspectValue>,
    count: usize,
) -> Link {
    if count == 0 {
        return None;
    }
    let left_count = count / 2;
    let left = sorted_node(values, left_count);
    let value = values
        .next()
        .expect("the admitted stream has its declared aspect count");
    let right = sorted_node(values, count - left_count - 1);
    Some(node(value, left, right))
}

fn height(link: &Link) -> usize {
    link.as_ref().map_or(0, |node| node.height)
}

fn length(link: &Link) -> usize {
    link.as_ref().map_or(0, |node| node.len)
}

fn node(value: ContractValidatedAspectValue, left: Link, right: Link) -> Arc<EntryNode> {
    Arc::new(EntryNode {
        value,
        height: 1 + height(&left).max(height(&right)),
        len: 1 + length(&left) + length(&right),
        left,
        right,
        birth_seq: crate::aspects::allocation_visitor::next_allocation_birth_seq(),
    })
}

fn balanced(value: ContractValidatedAspectValue, left: Link, right: Link) -> Arc<EntryNode> {
    if height(&left) > height(&right) + 1 {
        let root = left.as_ref().expect("left-heavy AVL node has a left child");
        if height(&root.left) >= height(&root.right) {
            return node(
                root.value.clone(),
                root.left.clone(),
                Some(node(value, root.right.clone(), right)),
            );
        }
        let pivot = root
            .right
            .as_ref()
            .expect("left-right AVL rotation has its pivot");
        return node(
            pivot.value.clone(),
            Some(node(
                root.value.clone(),
                root.left.clone(),
                pivot.left.clone(),
            )),
            Some(node(value, pivot.right.clone(), right)),
        );
    }
    if height(&right) > height(&left) + 1 {
        let root = right
            .as_ref()
            .expect("right-heavy AVL node has a right child");
        if height(&root.right) >= height(&root.left) {
            return node(
                root.value.clone(),
                Some(node(value, left, root.left.clone())),
                root.right.clone(),
            );
        }
        let pivot = root
            .left
            .as_ref()
            .expect("right-left AVL rotation has its pivot");
        return node(
            pivot.value.clone(),
            Some(node(value, left, pivot.left.clone())),
            Some(node(
                root.value.clone(),
                pivot.right.clone(),
                root.right.clone(),
            )),
        );
    }
    node(value, left, right)
}

fn insert_node(
    root: &Link,
    value: ContractValidatedAspectValue,
) -> (Arc<EntryNode>, Option<ContractValidatedAspectValue>) {
    let Some(root) = root else {
        return (node(value, None, None), None);
    };
    match value.key().cmp(root.value.key()) {
        std::cmp::Ordering::Less => {
            let (left, previous) = insert_node(&root.left, value);
            (
                balanced(root.value.clone(), Some(left), root.right.clone()),
                previous,
            )
        }
        std::cmp::Ordering::Equal => (
            node(value, root.left.clone(), root.right.clone()),
            Some(root.value.clone()),
        ),
        std::cmp::Ordering::Greater => {
            let (right, previous) = insert_node(&root.right, value);
            (
                balanced(root.value.clone(), root.left.clone(), Some(right)),
                previous,
            )
        }
    }
}

fn remove_node(root: &Link, key: &AspectKey) -> (Link, Option<ContractValidatedAspectValue>) {
    let Some(root) = root else {
        return (None, None);
    };
    match key.cmp(root.value.key()) {
        std::cmp::Ordering::Less => {
            let (left, removed) = remove_node(&root.left, key);
            if removed.is_none() {
                return (Some(Arc::clone(root)), None);
            }
            (
                Some(balanced(root.value.clone(), left, root.right.clone())),
                removed,
            )
        }
        std::cmp::Ordering::Greater => {
            let (right, removed) = remove_node(&root.right, key);
            if removed.is_none() {
                return (Some(Arc::clone(root)), None);
            }
            (
                Some(balanced(root.value.clone(), root.left.clone(), right)),
                removed,
            )
        }
        std::cmp::Ordering::Equal => {
            let removed = Some(root.value.clone());
            let next = match (&root.left, &root.right) {
                (None, _) => root.right.clone(),
                (_, None) => root.left.clone(),
                (_, Some(right)) => {
                    let (next_min, right) = remove_min(right);
                    Some(balanced(next_min.value.clone(), root.left.clone(), right))
                }
            };
            (next, removed)
        }
    }
}

fn remove_min(root: &Arc<EntryNode>) -> (&EntryNode, Link) {
    match &root.left {
        None => (root, root.right.clone()),
        Some(left) => {
            let (minimum, left) = remove_min(left);
            (
                minimum,
                Some(balanced(root.value.clone(), left, root.right.clone())),
            )
        }
    }
}

// Every two AVL levels at least double the minimum population plus one.
// An addressable tree therefore fits within twice the usize bit width.
const STACK: usize = 2 * usize::BITS as usize;

pub(super) struct Entries<'a> {
    pending: [Option<&'a EntryNode>; STACK],
    depth: usize,
    remaining: usize,
}

impl<'a> Entries<'a> {
    fn descend(&mut self, mut current: Option<&'a EntryNode>) {
        while let Some(node) = current {
            self.pending[self.depth] = Some(node);
            self.depth += 1;
            current = node.left.as_deref();
        }
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = (&'a AspectKey, &'a ContractValidatedAspectValue);
    fn next(&mut self) -> Option<Self::Item> {
        self.depth = self.depth.checked_sub(1)?;
        let node = self.pending[self.depth]
            .take()
            .expect("pending AVL traversal node");
        self.descend(node.right.as_deref());
        self.remaining -= 1;
        Some((node.value.key(), &node.value))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for Entries<'_> {}

impl PartialEq for AspectEntryTree {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}
impl Eq for AspectEntryTree {}

impl std::fmt::Debug for AspectEntryTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl Serialize for AspectEntryTree {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (key, value) in self.iter() {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}
