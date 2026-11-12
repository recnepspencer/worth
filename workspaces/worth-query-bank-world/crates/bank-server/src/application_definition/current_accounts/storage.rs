//! Controlled finite NodeWriter adapter; comparison borrows no Source image.
use super::provider::{Denial, Node, Stop};
use std::cmp::Ordering;
use worth_query_host::facade::{primary_graph::*, runtime::ExecutionOrderedTreeStorage};
pub(super) struct Storage<'nodes, 'call, 'control> {
    pub nodes: &'call mut WorthQueryManagedIndexNodeWriter<'nodes, Node>,
    pub control: &'call WorthQueryManagedIndexControl<'control>,
}
impl ExecutionOrderedTreeStorage for Storage<'_, '_, '_> {
    type Id = WorthQueryManagedIndexNodeId;
    type Source = WorthQueryManagedIndexSourceSlot;
    type Stop = Stop;
    fn checkpoint(&mut self) -> Result<(), Stop> {
        self.control.checkpoint(1).map_err(Into::into)
    }
    fn compare_sources(
        &mut self,
        left: Self::Source,
        right: Self::Source,
    ) -> Result<Ordering, Stop> {
        self.checkpoint()?;
        Ok(left.cmp(&right))
    }
    fn append(&mut self, node: Node) -> Result<Self::Id, Stop> {
        self.nodes.push(node).map_err(Into::into)
    }
    fn read(&mut self, id: Self::Id) -> Result<Node, Stop> {
        self.nodes.get(id).map(|node| *node).map_err(Into::into)
    }
    fn replace(&mut self, id: Self::Id, node: Node) -> Result<(), Stop> {
        self.nodes.replace(id, node).map_err(Into::into)
    }
    fn remove(&mut self, id: Self::Id) -> Result<(), Stop> {
        self.nodes.remove(id).map_err(Into::into)
    }
    fn invalid(&self) -> Stop {
        Stop::Domain(Denial::InvalidTree)
    }
}
