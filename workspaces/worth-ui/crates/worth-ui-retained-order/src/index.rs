use std::collections::HashMap;
use std::hash::Hash;

type Link = Option<usize>;

use crate::cost::CostTracker;

#[path = "index/ordered.rs"]
mod ordered;
#[path = "index/weight.rs"]
mod weight;

pub(crate) use ordered::Ordered;

pub(crate) struct BoundedOrderIndex<Identity> {
    capacity: usize,
    nodes: Vec<Node<Identity>>,
    free: Vec<usize>,
    root: Link,
    identities: HashMap<Identity, usize>,
    cost: CostTracker,
    high_water_entries: usize,
}

struct Node<Identity> {
    identity: Option<Identity>,
    weight: u32,
    subtree_max_weight: u32,
    left: Link,
    right: Link,
    parent: Link,
    height: u16,
    size: usize,
}

struct Removal<Identity> {
    physical: usize,
    moved: Option<(Identity, usize)>,
}

impl<Identity> BoundedOrderIndex<Identity>
where
    Identity: Copy + Eq + Hash,
{
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            capacity,
            nodes: Vec::new(),
            free: Vec::new(),
            root: None,
            identities: HashMap::new(),
            cost: CostTracker::new(),
            high_water_entries: 0,
        }
    }

    pub(crate) fn contains(&self, identity: Identity) -> bool {
        self.cost.identity_lookup();
        self.identities.contains_key(&identity)
    }

    pub(crate) fn insert_at(
        &mut self,
        rank: usize,
        identity: Identity,
    ) -> Result<(), crate::UiRetainedOrderDenial> {
        self.insert_at_weighted(rank, identity, 0)
    }

    pub(crate) fn insert_at_weighted(
        &mut self,
        rank: usize,
        identity: Identity,
        weight: u32,
    ) -> Result<(), crate::UiRetainedOrderDenial> {
        if self.len() >= self.capacity {
            return Err(crate::UiRetainedOrderDenial::CapacityExceeded);
        }
        if rank > self.len() {
            return Err(crate::UiRetainedOrderDenial::InvalidRank);
        }
        if self.contains(identity) {
            return Err(crate::UiRetainedOrderDenial::DuplicateIdentity);
        }
        let node = self.allocate(identity, weight);
        self.root = Some(self.insert_node(self.root, node, rank));
        self.set_parent(self.root, None);
        self.identities.insert(identity, node);
        self.high_water_entries = self.high_water_entries.max(self.len());
        Ok(())
    }

    pub(crate) fn remove(&mut self, identity: Identity) -> bool {
        self.cost.identity_lookup();
        let Some(node) = self.identities.get(&identity).copied() else {
            return false;
        };
        let rank = self.rank_of_node(node);
        let (root, removal) = self.remove_at(self.root.expect("indexed identity has a root"), rank);
        self.root = root;
        self.set_parent(self.root, None);
        self.identities.remove(&identity);
        if let Some((moved, target)) = removal.moved {
            self.identities.insert(moved, target);
        }
        self.release(removal.physical);
        true
    }

    pub(crate) fn rank(&self, identity: Identity) -> Option<usize> {
        self.cost.identity_lookup();
        self.identities
            .get(&identity)
            .map(|node| self.rank_of_node(*node))
    }

    pub(crate) fn identity_at(&self, mut rank: usize) -> Option<Identity> {
        let mut current = self.root?;
        loop {
            self.cost.node_touch();
            let left_size = self.link_size(self.nodes[current].left);
            if rank < left_size {
                current = self.nodes[current].left?;
            } else if rank == left_size {
                return self.nodes[current].identity;
            } else {
                rank -= left_size + 1;
                current = self.nodes[current].right?;
            }
        }
    }

    pub(crate) fn ordered(&self) -> Ordered<'_, Identity> {
        Ordered::new(self)
    }

    pub(crate) fn len(&self) -> usize {
        self.link_size(self.root)
    }

    pub(crate) fn height(&self) -> usize {
        usize::from(self.link_height(self.root))
    }

    pub(crate) fn take_cost(&self) -> crate::UiRetainedOrderCost {
        self.cost
            .take(self.len(), self.nodes.len(), self.high_water_entries)
    }

    fn allocate(&mut self, identity: Identity, weight: u32) -> usize {
        if let Some(index) = self.free.pop() {
            self.nodes[index] = Node::new(identity, weight);
            index
        } else {
            self.nodes.push(Node::new(identity, weight));
            self.nodes.len() - 1
        }
    }

    fn release(&mut self, index: usize) {
        self.nodes[index].identity = None;
        self.nodes[index].weight = 0;
        self.nodes[index].subtree_max_weight = 0;
        self.nodes[index].left = None;
        self.nodes[index].right = None;
        self.nodes[index].parent = None;
        self.nodes[index].height = 0;
        self.nodes[index].size = 0;
        self.free.push(index);
    }

    fn insert_node(&mut self, root: Link, node: usize, rank: usize) -> usize {
        let Some(root) = root else {
            return node;
        };
        self.cost.node_touch();
        let left_size = self.link_size(self.nodes[root].left);
        if rank <= left_size {
            let child = self.insert_node(self.nodes[root].left, node, rank);
            self.nodes[root].left = Some(child);
            self.nodes[child].parent = Some(root);
        } else {
            let child = self.insert_node(self.nodes[root].right, node, rank - left_size - 1);
            self.nodes[root].right = Some(child);
            self.nodes[child].parent = Some(root);
        }
        self.rebalance(root)
    }

    fn remove_at(&mut self, root: usize, rank: usize) -> (Link, Removal<Identity>) {
        self.cost.node_touch();
        let left_size = self.link_size(self.nodes[root].left);
        if rank < left_size {
            let left = self.nodes[root]
                .left
                .expect("rank is inside the left subtree");
            let (child, removal) = self.remove_at(left, rank);
            self.nodes[root].left = child;
            self.set_parent(child, Some(root));
            return (Some(self.rebalance(root)), removal);
        }
        if rank > left_size {
            let right = self.nodes[root]
                .right
                .expect("rank is inside the right subtree");
            let (child, removal) = self.remove_at(right, rank - left_size - 1);
            self.nodes[root].right = child;
            self.set_parent(child, Some(root));
            return (Some(self.rebalance(root)), removal);
        }
        self.remove_root(root)
    }

    fn remove_root(&mut self, root: usize) -> (Link, Removal<Identity>) {
        match (self.nodes[root].left, self.nodes[root].right) {
            (None, child) | (child, None) => {
                self.set_parent(child, self.nodes[root].parent);
                (
                    child,
                    Removal {
                        physical: root,
                        moved: None,
                    },
                )
            }
            (Some(_), Some(right)) => {
                let successor = self.minimum(right);
                let moved = self.nodes[successor]
                    .identity
                    .expect("an active successor has an identity");
                self.nodes[root].identity = Some(moved);
                self.nodes[root].weight = self.nodes[successor].weight;
                let (new_right, removal) = self.remove_at(right, 0);
                self.nodes[root].right = new_right;
                self.set_parent(new_right, Some(root));
                let balanced = self.rebalance(root);
                (
                    Some(balanced),
                    Removal {
                        physical: removal.physical,
                        moved: Some((moved, root)),
                    },
                )
            }
        }
    }

    fn minimum(&self, mut node: usize) -> usize {
        self.cost.node_touch();
        while let Some(left) = self.nodes[node].left {
            self.cost.node_touch();
            node = left;
        }
        node
    }

    fn rank_of_node(&self, mut node: usize) -> usize {
        self.cost.node_touch();
        let mut rank = self.link_size(self.nodes[node].left);
        while let Some(parent) = self.nodes[node].parent {
            self.cost.node_touch();
            if self.nodes[parent].right == Some(node) {
                rank += self.link_size(self.nodes[parent].left) + 1;
            }
            node = parent;
        }
        rank
    }

    fn rebalance(&mut self, root: usize) -> usize {
        self.refresh(root);
        let balance = self.balance(root);
        if balance > 1 {
            let left = self.nodes[root]
                .left
                .expect("left-heavy node has a left child");
            if self.balance(left) < 0 {
                let rotated = self.rotate_left(left);
                self.nodes[root].left = Some(rotated);
                self.nodes[rotated].parent = Some(root);
            }
            return self.rotate_right(root);
        }
        if balance < -1 {
            let right = self.nodes[root]
                .right
                .expect("right-heavy node has a right child");
            if self.balance(right) > 0 {
                let rotated = self.rotate_right(right);
                self.nodes[root].right = Some(rotated);
                self.nodes[rotated].parent = Some(root);
            }
            return self.rotate_left(root);
        }
        root
    }

    fn rotate_left(&mut self, root: usize) -> usize {
        self.cost.rotation();
        let parent = self.nodes[root].parent;
        let pivot = self.nodes[root]
            .right
            .expect("left rotation has a right child");
        let middle = self.nodes[pivot].left;
        self.nodes[root].right = middle;
        self.set_parent(middle, Some(root));
        self.nodes[pivot].left = Some(root);
        self.nodes[root].parent = Some(pivot);
        self.nodes[pivot].parent = parent;
        self.refresh(root);
        self.refresh(pivot);
        pivot
    }

    fn rotate_right(&mut self, root: usize) -> usize {
        self.cost.rotation();
        let parent = self.nodes[root].parent;
        let pivot = self.nodes[root]
            .left
            .expect("right rotation has a left child");
        let middle = self.nodes[pivot].right;
        self.nodes[root].left = middle;
        self.set_parent(middle, Some(root));
        self.nodes[pivot].right = Some(root);
        self.nodes[root].parent = Some(pivot);
        self.nodes[pivot].parent = parent;
        self.refresh(root);
        self.refresh(pivot);
        pivot
    }

    fn refresh(&mut self, node: usize) {
        self.nodes[node].height = 1 + self
            .link_height(self.nodes[node].left)
            .max(self.link_height(self.nodes[node].right));
        self.nodes[node].size =
            1 + self.link_size(self.nodes[node].left) + self.link_size(self.nodes[node].right);
        self.nodes[node].subtree_max_weight = self.nodes[node]
            .weight
            .max(self.link_max_weight(self.nodes[node].left))
            .max(self.link_max_weight(self.nodes[node].right));
    }

    fn balance(&self, node: usize) -> i32 {
        i32::from(self.link_height(self.nodes[node].left))
            - i32::from(self.link_height(self.nodes[node].right))
    }

    fn link_height(&self, link: Link) -> u16 {
        link.map(|node| self.nodes[node].height).unwrap_or(0)
    }

    fn link_size(&self, link: Link) -> usize {
        link.map(|node| self.nodes[node].size).unwrap_or(0)
    }

    fn link_max_weight(&self, link: Link) -> u32 {
        link.map(|node| self.nodes[node].subtree_max_weight)
            .unwrap_or(0)
    }

    fn set_parent(&mut self, link: Link, parent: Link) {
        if let Some(node) = link {
            self.nodes[node].parent = parent;
        }
    }
}

impl<Identity> Node<Identity> {
    fn new(identity: Identity, weight: u32) -> Self {
        Self {
            identity: Some(identity),
            weight,
            subtree_max_weight: weight,
            left: None,
            right: None,
            parent: None,
            height: 1,
            size: 1,
        }
    }
}

#[cfg(test)]
#[path = "index_tests.rs"]
mod tests;
