//! Rebuildable canonical Patricia commitment. Shape depends only on keys, never
//! insertion order, allocation identity, balancing history, or retained roots.
use super::PartitionContentDigestError;
use crate::storage::substrate::{StorageAllocation as Arc, StorageAllocationVisitor};
use sha2::{Digest, Sha256};

mod append_frontier;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default)]
pub(super) struct DigestIndex {
    root: Option<Arc<Node>>,
    history_last_value: Option<Arc<HistoryLastValue>>,
}

#[derive(Debug, Clone)]
struct HistoryLastValue {
    key: u128,
    value: [u8; 32],
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
        debug_assert!(
            /* Proof: from_sorted_entries is private to commitment rebuilds: its callers sort the unique arena slot or adjacency slot keys before construction. Partial media frontiers use from_append_frontier validation instead. */
            entries.windows(2).all(|pair| pair[0].0 < pair[1].0)
        );
        Self {
            root: (!entries.is_empty()).then(|| build_sorted(entries)),
            history_last_value: None,
        }
    }

    pub(super) fn digest(&self) -> [u8; 32] {
        self.root
            .as_ref()
            .map_or_else(|| hash(0, &[]), |root| root.digest)
    }

    pub(super) fn set(
        &mut self,
        key: u128,
        value: Option<[u8; 32]>,
    ) -> Result<(), PartitionContentDigestError> {
        let changed = match value {
            Some(digest) => insert(self.root.take(), key, digest).map(Some),
            None => remove(self.root.take(), key),
        };
        match changed {
            Ok(root) => self.root = root,
            Err(root) => {
                self.root = Some(root);
                return Err(PartitionContentDigestError::InvalidHistoryCoverage);
            }
        }
        if self
            .history_last_value
            .as_ref()
            .is_some_and(|last| key >= last.key)
        {
            self.history_last_value = value.map(|value| Arc::new(HistoryLastValue { key, value }));
        }
        Ok(())
    }

    pub(super) fn allocation_bytes(&self) -> u64 {
        self.root
            .as_ref()
            .map_or(0, |root| root.nodes as u64 * Arc::<Node>::layout_bytes())
            + self
                .history_last_value
                .as_ref()
                .map_or(0, |_| Arc::<HistoryLastValue>::layout_bytes())
    }

    pub(super) fn visit_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        if let Some(last) = &self.history_last_value {
            visitor.visit(last.observation(false));
        }
        fn walk(node: &Arc<Node>, visitor: &mut dyn StorageAllocationVisitor) {
            if visitor.visit(node.observation(false))
                && let Some(children) = &node.children
            {
                for child in children {
                    walk(child, visitor);
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
    Arc::new(Node::branch(
        bit,
        [
            build_sorted(&entries[..split]),
            build_sorted(&entries[split..]),
        ],
    ))
}

// An Err retains the original root. Opaque checkpoint siblings may be carried,
// but only an admitted Full image can expose their contents for edits.
fn insert(root: Option<Arc<Node>>, key: u128, digest: [u8; 32]) -> Result<Arc<Node>, Arc<Node>> {
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
        return Ok(leaf());
    };
    let differing_bit = (root.key ^ key).leading_zeros();
    if differing_bit < root.bit {
        let next = leaf();
        let children = if direction(key, differing_bit) == 0 {
            [next, root]
        } else {
            [root, next]
        };
        return Ok(Arc::new(Node::branch(differing_bit, children)));
    }
    if root.bit == 128 {
        return Ok(leaf());
    }
    let Some(children) = &root.children else {
        return Err(root);
    };
    let side = direction(key, root.bit);
    let changed = match insert(Some(children[side].clone()), key, digest) {
        Ok(changed) => changed,
        Err(_) => return Err(root),
    };
    let node = Arc::make_mut(&mut root);
    let Some(children) = &mut node.children else {
        return Err(root);
    };
    children[side] = changed;
    node.key = children[0].key;
    node.nodes = 1 + children[0].nodes + children[1].nodes;
    node.digest = branch_digest(node.bit, children);
    Ok(root)
}

fn remove(root: Option<Arc<Node>>, key: u128) -> Result<Option<Arc<Node>>, Arc<Node>> {
    let Some(mut root) = root else {
        return Ok(None);
    };
    if (root.key ^ key).leading_zeros() < root.bit {
        return Ok(Some(root));
    }
    if root.bit == 128 {
        return Ok(None);
    }
    let Some(children) = &root.children else {
        return Err(root);
    };
    let side = direction(key, root.bit);
    let changed = match remove(Some(children[side].clone()), key) {
        Ok(changed) => changed,
        Err(_) => return Err(root),
    };
    if let Some(changed) = changed {
        let node = Arc::make_mut(&mut root);
        let Some(children) = &mut node.children else {
            return Err(root);
        };
        children[side] = changed;
        node.key = children[0].key;
        node.nodes = 1 + children[0].nodes + children[1].nodes;
        node.digest = branch_digest(node.bit, children);
        Ok(Some(root))
    } else {
        Ok(Some(children[1 - side].clone()))
    }
}

impl Node {
    fn branch(bit: u32, children: [Arc<Node>; 2]) -> Self {
        Self {
            key: children[0].key,
            bit,
            digest: branch_digest(bit, &children),
            nodes: 1 + children[0].nodes + children[1].nodes,
            children: Some(children),
        }
    }
}
fn branch_digest(bit: u32, children: &[Arc<Node>; 2]) -> [u8; 32] {
    hash(
        2,
        &[&bit.to_be_bytes(), &children[0].digest, &children[1].digest],
    )
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
