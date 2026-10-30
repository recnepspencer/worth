//! Every observed write has already changed the isolated framework-owned backing.
use super::*;
use worth_query_consumer_values::BoxEditStorage;

pub(super) struct Edit<'call, 'node, 'control> {
    pub nodes: &'call mut WorthQueryManagedIndexNodeWriter<'node, Node>,
    pub control: &'call WorthQueryManagedIndexControl<'control>,
    pub calls: &'call Calls,
}
pub(super) fn invalid() -> Stop {
    Stop::Domain(BoxIndexDenial::InvalidTopology)
}
impl Edit<'_, '_, '_> {
    fn observe_write(&self) {
        let observer = self.calls.after_node_write.lock().unwrap().take();
        if let Some(observer) = observer {
            observer();
        }
    }
}
impl BoxEditStorage for Edit<'_, '_, '_> {
    type Id = WorthQueryManagedIndexNodeId;
    type Source = WorthQueryManagedIndexSourceSlot;
    type Stop = Stop;
    fn checkpoint(&mut self) -> Result<(), Stop> {
        self.control.checkpoint(1).map_err(Into::into)
    }
    fn read(&mut self, id: Self::Id) -> Result<Node, Stop> {
        self.calls.navigation.fetch_add(1, Ordering::SeqCst);
        self.nodes.get(id).map_err(Into::into)
    }
    fn append(&mut self, node: Node) -> Result<Self::Id, Stop> {
        let id = self.nodes.push(node)?;
        self.observe_write();
        Ok(id)
    }
    fn replace(&mut self, id: Self::Id, node: Node) -> Result<(), Stop> {
        self.nodes.replace(id, node)?;
        self.observe_write();
        Ok(())
    }
    fn remove(&mut self, id: Self::Id) -> Result<(), Stop> {
        self.nodes.remove(id)?;
        self.observe_write();
        Ok(())
    }
    fn invalid(&self) -> Stop {
        invalid()
    }
}
