//! Provider-exposed physical topology, separate from matching or a coverage witness.
use super::super::{
    WorthQueryManagedIndexNodeId as NodeId, WorthQueryManagedIndexSourceSlot as Slot,
};

/// One node's own Source occurrence and actual binary children.
/// Query checks the whole retained correspondence; construction grants no authority.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryManagedIndexNodeRegion {
    pub(super) source: Option<Slot>,
    pub(super) children: [Option<NodeId>; 2],
}
impl WorthQueryManagedIndexNodeRegion {
    pub const fn new(source: Option<Slot>, children: [Option<NodeId>; 2]) -> Self {
        Self { source, children }
    }
    pub const fn source(&self) -> Option<Slot> {
        self.source
    }
    pub const fn children(&self) -> [Option<NodeId>; 2] {
        self.children
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct Region {
    pub source: Option<Slot>,
    pub children: [Option<usize>; 2],
}
pub(super) struct Edit {
    pub ordinal: usize,
    pub before: Option<Region>,
    pub region: Option<Region>,
}
