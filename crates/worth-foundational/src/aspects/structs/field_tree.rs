//! Immutable struct fields: path-copying AVL updates and allocation-free ordered reads.
//! This is value backing, independent of runtime storage allocation authority.
use super::FieldKey;
use crate::values::AspectValue;
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::sync::Arc;

type Link = Option<Arc<Node>>;
#[derive(Clone)]
struct FieldEntry {
    key: Arc<FieldKey>,
    key_birth_seq: Option<u64>,
    value: Arc<AspectValue>,
    value_birth_seq: Option<u64>,
}
impl FieldEntry {
    fn fresh(key: FieldKey, value: AspectValue) -> Self {
        Self {
            key: Arc::new(key),
            key_birth_seq: crate::aspects::allocation_visitor::next_allocation_birth_seq(),
            value: Arc::new(value),
            value_birth_seq: crate::aspects::allocation_visitor::next_allocation_birth_seq(),
        }
    }
    #[cfg(test)]
    fn with_arc_value(
        key: FieldKey,
        value: Arc<AspectValue>,
        value_birth_seq: Option<u64>,
    ) -> Self {
        Self {
            key: Arc::new(key),
            key_birth_seq: crate::aspects::allocation_visitor::next_allocation_birth_seq(),
            value,
            value_birth_seq,
        }
    }
}
struct Node {
    entry: FieldEntry,
    left: Link,
    right: Link,
    height: usize,
    len: usize,
    birth_seq: Option<u64>,
}
#[derive(Clone, Default)]
pub(super) struct StructFields {
    root: Link,
}
impl StructFields {
    pub(super) fn new() -> Self {
        Self::default()
    }
    /// The caller has proved strict order. Consume the canonical stream once;
    /// each field obtains one Node, key and value Arc, without path-copy churn.
    pub(super) fn from_sorted_fields(
        mut fields: std::vec::IntoIter<(FieldKey, AspectValue)>,
    ) -> Self {
        let count = fields.len();
        Self {
            root: sorted_node(&mut fields, count),
        }
    }

    pub(super) fn len(&self) -> usize {
        length(&self.root)
    }
    pub(super) fn is_empty(&self) -> bool {
        self.root.is_none()
    }
    pub(super) fn contains_key(&self, key: &FieldKey) -> bool {
        self.get(key).is_some()
    }
    pub(super) fn get(&self, key: &FieldKey) -> Option<&Arc<AspectValue>> {
        self.get_key_value(key).map(|(_, value)| value)
    }
    pub(super) fn get_key_value(&self, key: &FieldKey) -> Option<(&FieldKey, &Arc<AspectValue>)> {
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            match key.cmp(&node.entry.key) {
                std::cmp::Ordering::Less => current = node.left.as_deref(),
                std::cmp::Ordering::Equal => return Some((&node.entry.key, &node.entry.value)),
                std::cmp::Ordering::Greater => current = node.right.as_deref(),
            }
        }
        None
    }
    pub(super) fn insert_fresh(&mut self, key: FieldKey, value: AspectValue) {
        self.root = Some(insert(&self.root, FieldEntry::fresh(key, value)));
    }
    #[cfg(test)]
    pub(super) fn insert(&mut self, key: FieldKey, value: Arc<AspectValue>) {
        self.root = Some(insert(
            &self.root,
            FieldEntry::with_arc_value(key, value, None),
        ));
    }
    pub(super) fn remove(&mut self, key: &FieldKey) {
        if self.contains_key(key) {
            self.root = remove(&self.root, key);
        }
    }
    pub(super) fn iter(&self) -> Fields<'_> {
        let mut fields = Fields {
            pending: [None; STACK],
            depth: 0,
            remaining: self.len(),
        };
        fields.descend(self.root.as_deref());
        fields
    }
    /// One node, key and value allocation per field; strings are quoted separately.
    pub(super) fn allocation_bytes_per_field() -> usize {
        Self::checked_allocation_bytes_per_field()
            .expect("the three fixed concrete field Arc layouts fit the address space")
    }

    pub(super) fn checked_allocation_bytes_per_field() -> Option<usize> {
        arc_storage::<Node>()?
            .checked_add(arc_storage::<FieldKey>()?)?
            .checked_add(arc_storage::<AspectValue>()?)
    }

    pub(super) fn edit_storage_bytes(
        &self,
        operations: usize,
        added_fields: usize,
    ) -> Option<usize> {
        let maximum = self.len().checked_add(added_fields)?.checked_add(1)?;
        // AVL height is less than twice the binary height. A double rotation
        // allocates at most three nodes at a reached level. Including key/value
        // Arc cells in each node bound is deliberately conservative.
        let levels = (usize::BITS as usize - maximum.leading_zeros() as usize)
            .checked_mul(2)?
            .checked_add(1)?;
        Self::checked_allocation_bytes_per_field()?
            .checked_mul(3)?
            .checked_mul(levels)?
            .checked_mul(operations)
    }
}
fn arc_storage<T>() -> Option<usize> {
    let (layout, _) = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .ok()?;
    Some(layout.pad_to_align().size())
}

fn sorted_node(fields: &mut std::vec::IntoIter<(FieldKey, AspectValue)>, count: usize) -> Link {
    if count == 0 {
        return None;
    }
    let left_count = count / 2;
    let left = sorted_node(fields, left_count);
    let (key, value) = fields
        .next()
        .expect("the admitted stream has its declared field count");
    let right = sorted_node(fields, count - left_count - 1);
    Some(node(FieldEntry::fresh(key, value), left, right))
}

fn height(link: &Link) -> usize {
    link.as_ref().map_or(0, |node| node.height)
}
fn length(link: &Link) -> usize {
    link.as_ref().map_or(0, |node| node.len)
}
fn node(entry: FieldEntry, left: Link, right: Link) -> Arc<Node> {
    Arc::new(Node {
        entry,
        height: 1 + height(&left).max(height(&right)),
        len: 1 + length(&left) + length(&right),
        left,
        right,
        birth_seq: crate::aspects::allocation_visitor::next_allocation_birth_seq(),
    })
}
fn balanced(entry: FieldEntry, left: Link, right: Link) -> Arc<Node> {
    if height(&left) > height(&right) + 1 {
        let root = left.as_ref().expect("left-heavy AVL node has a left child");
        if height(&root.left) >= height(&root.right) {
            return node(
                root.entry.clone(),
                root.left.clone(),
                Some(node(entry, root.right.clone(), right)),
            );
        }
        let pivot = root
            .right
            .as_ref()
            .expect("left-right AVL rotation has its pivot");
        return node(
            pivot.entry.clone(),
            Some(node(
                root.entry.clone(),
                root.left.clone(),
                pivot.left.clone(),
            )),
            Some(node(entry, pivot.right.clone(), right)),
        );
    }
    if height(&right) > height(&left) + 1 {
        let root = right
            .as_ref()
            .expect("right-heavy AVL node has a right child");
        if height(&root.right) >= height(&root.left) {
            return node(
                root.entry.clone(),
                Some(node(entry, left, root.left.clone())),
                root.right.clone(),
            );
        }
        let pivot = root
            .left
            .as_ref()
            .expect("right-left AVL rotation has its pivot");
        return node(
            pivot.entry.clone(),
            Some(node(entry, left, pivot.left.clone())),
            Some(node(
                root.entry.clone(),
                pivot.right.clone(),
                root.right.clone(),
            )),
        );
    }
    node(entry, left, right)
}
fn insert(root: &Link, entry: FieldEntry) -> Arc<Node> {
    let Some(root) = root else {
        return node(entry, None, None);
    };
    match entry.key.cmp(&root.entry.key) {
        std::cmp::Ordering::Less => balanced(
            root.entry.clone(),
            Some(insert(&root.left, entry)),
            root.right.clone(),
        ),
        std::cmp::Ordering::Equal => node(
            FieldEntry {
                key: root.entry.key.clone(),
                key_birth_seq: root.entry.key_birth_seq,
                value: entry.value,
                value_birth_seq: entry.value_birth_seq,
            },
            root.left.clone(),
            root.right.clone(),
        ),
        std::cmp::Ordering::Greater => balanced(
            root.entry.clone(),
            root.left.clone(),
            Some(insert(&root.right, entry)),
        ),
    }
}
fn remove(root: &Link, key: &FieldKey) -> Link {
    let root = root.as_ref()?;
    Some(match key.cmp(&root.entry.key) {
        std::cmp::Ordering::Less => balanced(
            root.entry.clone(),
            remove(&root.left, key),
            root.right.clone(),
        ),
        std::cmp::Ordering::Greater => balanced(
            root.entry.clone(),
            root.left.clone(),
            remove(&root.right, key),
        ),
        std::cmp::Ordering::Equal => match (&root.left, &root.right) {
            (None, _) => return root.right.clone(),
            (_, None) => return root.left.clone(),
            (_, Some(right)) => {
                let (next, right) = remove_min(right);
                balanced(next.entry.clone(), root.left.clone(), right)
            }
        },
    })
}
fn remove_min(root: &Arc<Node>) -> (&Node, Link) {
    match &root.left {
        None => (root, root.right.clone()),
        Some(left) => {
            let (minimum, left) = remove_min(left);
            (
                minimum,
                Some(balanced(root.entry.clone(), left, root.right.clone())),
            )
        }
    }
}
// Every two AVL levels at least double the minimum population plus one.
// An addressable tree therefore fits within twice the usize bit width.
const STACK: usize = 2 * usize::BITS as usize;
pub(super) struct Fields<'a> {
    pending: [Option<&'a Node>; STACK],
    depth: usize,
    remaining: usize,
}
impl<'a> Fields<'a> {
    fn descend(&mut self, mut current: Option<&'a Node>) {
        while let Some(node) = current {
            self.pending[self.depth] = Some(node);
            self.depth += 1;
            current = node.left.as_deref();
        }
    }
}
impl<'a> Iterator for Fields<'a> {
    type Item = (&'a FieldKey, &'a Arc<AspectValue>);
    fn next(&mut self) -> Option<Self::Item> {
        self.depth = self.depth.checked_sub(1)?;
        let node = self.pending[self.depth]
            .take()
            .expect("pending AVL traversal node");
        self.descend(node.right.as_deref());
        self.remaining -= 1;
        Some((&node.entry.key, &node.entry.value))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl ExactSizeIterator for Fields<'_> {}
impl PartialEq for StructFields {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}
impl Eq for StructFields {}
impl std::fmt::Debug for StructFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
impl Serialize for StructFields {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (key, value) in self.iter() {
            map.serialize_entry(key, value.as_ref())?;
        }
        map.end()
    }
}

mod allocation;

#[cfg(test)]
mod tests;
