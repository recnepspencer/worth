//! Incremental reachability for an issued DAG. Forming children pin a parent
//! before their installed edge is visible. The owner supplies graph custody.
use std::hash::Hash;

mod dirty_list;
pub use dirty_list::{DagDirtyEdit, DagDirtyLinks, DagDirtyList};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DagReferenceCount(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DagReferenceOverflow;

impl DagReferenceCount {
    pub fn acquire(&mut self) -> Result<(), DagReferenceOverflow> {
        self.0 = self.0.checked_add(1).ok_or(DagReferenceOverflow)?;
        Ok(())
    }

    pub fn release(&mut self) {
        self.0 = self.0.checked_sub(1).expect("each reference releases once");
    }

    pub const fn count(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DagReachability<NodeId: Clone + Ord + Hash> {
    dependencies: DagReferenceCount,
    forming: DagReferenceCount,
    first_child: Option<NodeId>,
}

impl<NodeId: Clone + Ord + Hash> Default for DagReachability<NodeId> {
    fn default() -> Self {
        Self {
            dependencies: DagReferenceCount::default(),
            forming: DagReferenceCount::default(),
            first_child: None,
        }
    }
}

impl<NodeId: Clone + Ord + Hash> DagReachability<NodeId> {
    pub fn reserve_child(&mut self) -> Result<(), DagReferenceOverflow> {
        self.dependencies.acquire()?;
        // The forming children are a subset of the dependencies.
        self.forming
            .acquire()
            .expect("forming count fits descendant count");
        Ok(())
    }

    pub fn settle_forming_child(&mut self) {
        self.forming.release();
    }

    pub fn release_dependency(&mut self) {
        self.dependencies.release();
    }

    pub const fn descendant_dependencies(&self) -> usize {
        self.dependencies.count()
    }

    pub const fn has_forming_child(&self) -> bool {
        self.forming.count() != 0
    }

    pub fn sole_installed_child(&self) -> Option<&NodeId> {
        (self.descendant_dependencies() == 1 && !self.has_forming_child())
            .then_some(self.first_child.as_ref())
            .flatten()
    }

    pub fn first_child(&self) -> Option<NodeId> {
        self.first_child.clone()
    }

    pub fn set_first_child(&mut self, child: Option<NodeId>) {
        self.first_child = child;
    }
}
