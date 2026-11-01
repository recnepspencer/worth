//! Edits consume actual isolated framework backing and predecessor Source slots.
use super::*;
use worth_query_consumer_values::BoxEditStorage;
pub(super) struct Edit<'a, 'n, 'c> {
    pub nodes: &'a mut WorthQueryManagedIndexNodeWriter<'n, Node>,
    pub control: &'a WorthQueryManagedIndexControl<'c>,
}
pub(super) fn invalid() -> Stop {
    Stop::Domain(BoxIndexDenial::InvalidTopology)
}
impl BoxEditStorage<ModelLocalMetre> for Edit<'_, '_, '_> {
    type Id = WorthQueryManagedIndexNodeId;
    type Source = WorthQueryManagedIndexSourceSlot;
    type Stop = Stop;
    fn checkpoint(&mut self) -> Result<(), Stop> {
        self.control.checkpoint(1).map_err(Into::into)
    }
    fn read(&mut self, id: Self::Id) -> Result<Node, Stop> {
        self.nodes.get(id).map_err(Into::into)
    }
    fn append(&mut self, node: Node) -> Result<Self::Id, Stop> {
        self.nodes.push(node).map_err(Into::into)
    }
    fn replace(&mut self, id: Self::Id, node: Node) -> Result<(), Stop> {
        if self.read(id)? == node {
            return self.control.poll().map_err(Into::into);
        }
        self.nodes.replace(id, node).map_err(Into::into)
    }
    fn remove(&mut self, id: Self::Id) -> Result<(), Stop> {
        self.nodes.remove(id).map_err(Into::into)
    }
    fn source_bounds(&mut self, source: Self::Source) -> Result<Bounds, Stop> {
        let id = self.nodes.before_source_node(source)?;
        let node = self.nodes.get(id)?;
        match node.links {
            BoxNodeLinks::Leaf { source: actual } if actual == source => Ok(node.bounds),
            _ => Err(invalid()),
        }
    }
    fn invalid(&self) -> Stop {
        invalid()
    }
}
