//! Rebuildable canonical Patricia commitment. Shape depends only on keys, never
//! insertion order, allocation identity, balancing history, or retained roots.
use crate::storage::substrate::{StorageAllocation as Arc, StorageAllocationVisitor};
use sha2::{Digest, Sha256};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default)]
pub(super) struct DigestIndex {
    root: Option<Arc<Node>>,
}

#[derive(Debug, Clone)]
struct Node {
    key: u128,
    bit: u32,
    digest: [u8; 32],
    children: Option<[Arc<Node>; 2]>,
    nodes: usize,
}

impl DigestIndex {
    /// Cold reconstruction of the same key-shaped Patricia commitment without
    /// copying an immutable path for every successive insertion.
    pub(super) fn from_sorted_entries(entries: &[(u128, [u8; 32])]) -> Self {
        debug_assert!(entries.windows(2).all(|pair| pair[0].0 < pair[1].0));
        Self {
            root: (!entries.is_empty()).then(|| build_sorted(entries)),
        }
    }

    pub(super) fn digest(&self) -> [u8; 32] {
        self.root
            .as_ref()
            .map_or_else(|| hash(0, &[]), |root| root.digest)
    }

    pub(super) fn set(&mut self, key: u128, value: Option<[u8; 32]>) {
        self.root = match value {
            Some(digest) => Some(insert(self.root.take(), key, digest)),
            None => remove(self.root.take(), key),
        };
    }

    pub(super) fn allocation_bytes(&self) -> u64 {
        self.root
            .as_ref()
            .map_or(0, |root| root.nodes as u64 * Arc::<Node>::layout_bytes())
    }

    pub(super) fn visit_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        fn walk(node: &Arc<Node>, visitor: &mut dyn StorageAllocationVisitor) {
            if visitor.visit(node.observation(false)) {
                if let Some(children) = &node.children {
                    for child in children {
                        walk(child, visitor);
                    }
                }
            }
        }
        if let Some(root) = &self.root {
            walk(root, visitor);
        }
    }
}

fn build_sorted(entries: &[(u128, [u8; 32])]) -> Arc<Node> {
    if let [(key, digest)] = entries {
        return Arc::new(Node {
            key: *key,
            bit: 128,
            digest: hash(1, &[&key.to_be_bytes(), digest]),
            children: None,
            nodes: 1,
        });
    }
    let bit = (entries[0].0 ^ entries[entries.len() - 1].0).leading_zeros();
    let split = entries.partition_point(|(key, _)| direction(*key, bit) == 0);
    let mut parent = Node {
        key: entries[0].0,
        bit,
        digest: [0; 32],
        children: Some([
            build_sorted(&entries[..split]),
            build_sorted(&entries[split..]),
        ]),
        nodes: 0,
    };
    refresh(&mut parent);
    Arc::new(parent)
}

fn insert(root: Option<Arc<Node>>, key: u128, digest: [u8; 32]) -> Arc<Node> {
    let leaf = || {
        Arc::new(Node {
            key,
            bit: 128,
            digest: hash(1, &[&key.to_be_bytes(), &digest]),
            children: None,
            nodes: 1,
        })
    };
    let Some(mut root) = root else {
        return leaf();
    };
    let differing_bit = (root.key ^ key).leading_zeros();
    if differing_bit < root.bit {
        let next = leaf();
        let children = if direction(key, differing_bit) == 0 {
            [next, root]
        } else {
            [root, next]
        };
        let mut parent = Node {
            key: children[0].key,
            bit: differing_bit,
            digest: [0; 32],
            children: Some(children),
            nodes: 0,
        };
        refresh(&mut parent);
        return Arc::new(parent);
    }
    if root.bit == 128 {
        return leaf();
    }
    let node = Arc::make_mut(&mut root);
    let side = direction(key, node.bit);
    let child = &mut node.children.as_mut().unwrap()[side];
    *child = insert(Some(child.clone()), key, digest);
    refresh(node);
    root
}

fn remove(root: Option<Arc<Node>>, key: u128) -> Option<Arc<Node>> {
    let mut root = root?;
    if (root.key ^ key).leading_zeros() < root.bit {
        return Some(root);
    }
    if root.bit == 128 {
        return None;
    }
    let side = direction(key, root.bit);
    let node = Arc::make_mut(&mut root);
    let children = node.children.as_mut().unwrap();
    if let Some(changed) = remove(Some(children[side].clone()), key) {
        children[side] = changed;
        refresh(node);
        Some(root)
    } else {
        Some(children[1 - side].clone())
    }
}

fn refresh(node: &mut Node) {
    let children = node.children.as_ref().unwrap();
    node.key = children[0].key;
    node.nodes = 1 + children[0].nodes + children[1].nodes;
    node.digest = hash(
        2,
        &[
            &node.bit.to_be_bytes(),
            &children[0].digest,
            &children[1].digest,
        ],
    );
}

fn direction(key: u128, bit: u32) -> usize {
    ((key >> (127 - bit)) & 1) as usize
}

fn hash(tag: u8, chunks: &[&[u8]]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"worth.relational.content-index.v1\0");
    hash.update([tag]);
    for chunk in chunks {
        hash.update(chunk);
    }
    hash.finalize().into()
}
