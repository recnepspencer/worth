//! Intrusive FIFO for changed graph rows. Enqueue and removal visit at most
//! two neighboring rows and allocate nothing after the owner reserves its row.
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DagDirtyLinks<NodeId> {
    neighbors: Option<(Option<NodeId>, Option<NodeId>)>,
}

impl<NodeId> Default for DagDirtyLinks<NodeId> {
    fn default() -> Self {
        Self { neighbors: None }
    }
}

impl<NodeId: Clone> DagDirtyLinks<NodeId> {
    pub const fn is_marked(&self) -> bool {
        self.neighbors.is_some()
    }

    pub fn next(&self) -> Option<NodeId> {
        self.neighbors.as_ref().and_then(|(_, next)| next.clone())
    }

    pub fn apply(&mut self, edit: DagDirtyEdit<NodeId>) {
        let (previous, next) = self.neighbors.as_mut().expect("linked dirty neighbor");
        match edit {
            DagDirtyEdit::Previous(node) => *previous = node,
            DagDirtyEdit::Next(node) => *next = node,
        }
    }
}

pub enum DagDirtyEdit<NodeId> {
    Previous(Option<NodeId>),
    Next(Option<NodeId>),
}

#[derive(Debug)]
pub struct DagDirtyList<NodeId: Clone + Ord + Hash> {
    head: Option<NodeId>,
    tail: Option<NodeId>,
    len: usize,
}

impl<NodeId: Clone + Ord + Hash> Default for DagDirtyList<NodeId> {
    fn default() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }
}

impl<NodeId: Clone + Ord + Hash> DagDirtyList<NodeId> {
    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn head(&self) -> Option<NodeId> {
        self.head.clone()
    }

    pub fn tail(&self) -> Option<NodeId> {
        self.tail.clone()
    }

    pub fn mark(
        &mut self,
        node: NodeId,
        links: &mut DagDirtyLinks<NodeId>,
        mut edit: impl FnMut(NodeId, DagDirtyEdit<NodeId>),
    ) {
        if links.is_marked() {
            return;
        }
        if let Some(tail) = self.tail.clone() {
            edit(tail, DagDirtyEdit::Next(Some(node.clone())));
        } else {
            self.head = Some(node.clone());
        }
        links.neighbors = Some((self.tail.clone(), None));
        self.tail = Some(node);
        self.len += 1;
    }

    pub fn forget(
        &mut self,
        links: &mut DagDirtyLinks<NodeId>,
        mut edit: impl FnMut(NodeId, DagDirtyEdit<NodeId>),
    ) {
        let Some((previous, next)) = links.neighbors.take() else {
            return;
        };
        if let Some(previous) = previous.clone() {
            edit(previous, DagDirtyEdit::Next(next.clone()));
        } else {
            self.head = next.clone();
        }
        if let Some(next) = next {
            edit(next, DagDirtyEdit::Previous(previous));
        } else {
            self.tail = previous;
        }
        self.len -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_marks_and_middle_removal_preserve_fifo_neighbors() {
        let mut list = DagDirtyList::default();
        let mut rows = [DagDirtyLinks::default(); 3];
        for node in [0, 1, 1, 2] {
            let mut links = rows[node];
            list.mark(node, &mut links, |id, edit| rows[id].apply(edit));
            rows[node] = links;
        }
        assert_eq!(list.len(), 3);

        for (removed, new_head) in [(1, Some(0)), (0, Some(2)), (2, None)] {
            let mut links = std::mem::take(&mut rows[removed]);
            list.forget(&mut links, |id, edit| rows[id].apply(edit));
            assert!(!links.is_marked());
            assert_eq!(list.head(), new_head);
        }
        assert_eq!(list.tail(), None);
        assert!(list.is_empty());
    }
}
