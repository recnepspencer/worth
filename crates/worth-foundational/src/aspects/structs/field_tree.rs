//! Immutable struct fields: path-copying AVL updates and allocation-free ordered reads.
//! This is value backing, independent of runtime storage allocation authority.
use super::FieldKey;
use crate::values::AspectValue;
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::sync::Arc;

type Link = Option<Arc<Node>>;
struct Node {
    key: Arc<FieldKey>,
    value: Arc<AspectValue>,
    left: Link,
    right: Link,
    height: usize,
    len: usize,
}
#[derive(Clone, Default)]
pub(super) struct StructFields {
    root: Link,
}
impl StructFields {
    pub(super) fn new() -> Self {
        Self::default()
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
            match key.cmp(&node.key) {
                std::cmp::Ordering::Less => current = node.left.as_deref(),
                std::cmp::Ordering::Equal => return Some((&node.key, &node.value)),
                std::cmp::Ordering::Greater => current = node.right.as_deref(),
            }
        }
        None
    }
    pub(super) fn insert(&mut self, key: FieldKey, value: Arc<AspectValue>) {
        self.root = Some(insert(&self.root, Arc::new(key), value));
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
        std::mem::size_of::<Node>()
            + std::mem::size_of::<FieldKey>()
            + std::mem::size_of::<AspectValue>()
            + 6 * std::mem::size_of::<usize>()
    }
}
fn height(link: &Link) -> usize {
    link.as_ref().map_or(0, |node| node.height)
}
fn length(link: &Link) -> usize {
    link.as_ref().map_or(0, |node| node.len)
}
fn node(key: Arc<FieldKey>, value: Arc<AspectValue>, left: Link, right: Link) -> Arc<Node> {
    Arc::new(Node {
        key,
        value,
        height: 1 + height(&left).max(height(&right)),
        len: 1 + length(&left) + length(&right),
        left,
        right,
    })
}
fn balanced(key: Arc<FieldKey>, value: Arc<AspectValue>, left: Link, right: Link) -> Arc<Node> {
    if height(&left) > height(&right) + 1 {
        let root = left.as_ref().expect("left-heavy AVL node has a left child");
        if height(&root.left) >= height(&root.right) {
            return node(
                root.key.clone(),
                root.value.clone(),
                root.left.clone(),
                Some(node(key, value, root.right.clone(), right)),
            );
        }
        let pivot = root
            .right
            .as_ref()
            .expect("left-right AVL rotation has its pivot");
        return node(
            pivot.key.clone(),
            pivot.value.clone(),
            Some(node(
                root.key.clone(),
                root.value.clone(),
                root.left.clone(),
                pivot.left.clone(),
            )),
            Some(node(key, value, pivot.right.clone(), right)),
        );
    }
    if height(&right) > height(&left) + 1 {
        let root = right
            .as_ref()
            .expect("right-heavy AVL node has a right child");
        if height(&root.right) >= height(&root.left) {
            return node(
                root.key.clone(),
                root.value.clone(),
                Some(node(key, value, left, root.left.clone())),
                root.right.clone(),
            );
        }
        let pivot = root
            .left
            .as_ref()
            .expect("right-left AVL rotation has its pivot");
        return node(
            pivot.key.clone(),
            pivot.value.clone(),
            Some(node(key, value, left, pivot.left.clone())),
            Some(node(
                root.key.clone(),
                root.value.clone(),
                pivot.right.clone(),
                root.right.clone(),
            )),
        );
    }
    node(key, value, left, right)
}
fn insert(root: &Link, key: Arc<FieldKey>, value: Arc<AspectValue>) -> Arc<Node> {
    let Some(root) = root else {
        return node(key, value, None, None);
    };
    match key.cmp(&root.key) {
        std::cmp::Ordering::Less => balanced(
            root.key.clone(),
            root.value.clone(),
            Some(insert(&root.left, key, value)),
            root.right.clone(),
        ),
        std::cmp::Ordering::Equal => node(
            root.key.clone(),
            value,
            root.left.clone(),
            root.right.clone(),
        ),
        std::cmp::Ordering::Greater => balanced(
            root.key.clone(),
            root.value.clone(),
            root.left.clone(),
            Some(insert(&root.right, key, value)),
        ),
    }
}
fn remove(root: &Link, key: &FieldKey) -> Link {
    let root = root.as_ref()?;
    Some(match key.cmp(&root.key) {
        std::cmp::Ordering::Less => balanced(
            root.key.clone(),
            root.value.clone(),
            remove(&root.left, key),
            root.right.clone(),
        ),
        std::cmp::Ordering::Greater => balanced(
            root.key.clone(),
            root.value.clone(),
            root.left.clone(),
            remove(&root.right, key),
        ),
        std::cmp::Ordering::Equal => match (&root.left, &root.right) {
            (None, _) => return root.right.clone(),
            (_, None) => return root.left.clone(),
            (_, Some(right)) => {
                let (next, right) = remove_min(right);
                balanced(
                    next.key.clone(),
                    next.value.clone(),
                    root.left.clone(),
                    right,
                )
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
                Some(balanced(
                    root.key.clone(),
                    root.value.clone(),
                    left,
                    root.right.clone(),
                )),
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
        Some((&node.key, &node.value))
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

#[cfg(test)]
mod tests;
