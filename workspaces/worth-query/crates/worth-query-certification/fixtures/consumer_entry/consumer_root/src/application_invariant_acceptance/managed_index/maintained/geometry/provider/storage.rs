//! Every observed write has already changed the isolated framework-owned backing.
use super::*;
use worth_query_consumer_values::BoxEditStorage;

pub(super) struct Edit<'call, 'node, 'control> {
    pub nodes: &'call mut WorthQueryManagedIndexNodeWriter<'node, Node>,
    pub control: &'call WorthQueryManagedIndexControl<'control>,
    pub calls: &'call Calls,
    pub observer: &'call Mutex<Option<Box<dyn FnOnce() + Send>>>,
}
pub(super) fn invalid() -> Stop {
    Stop::Domain(BoxIndexDenial::InvalidTopology)
}
impl Edit<'_, '_, '_> {
    fn observe_write(&self) {
        let observer = self.observer.lock().unwrap().take();
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
        // The actual writer validates the coordinate before exact data equality
        // can omit an ineffective backing edit. Source deltas remain independent.
        if self.read(id)? == node {
            return self.control.poll().map_err(Into::into);
        }
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
    fn source_bounds(
        &mut self,
        source: Self::Source,
    ) -> Result<worth_query_consumer_values::Box2, Stop> {
        // Read the predecessor leaf's key before this occurrence is edited.
        // Earlier edits only rotate branches; other leaf coordinates stay live.
        let id = self.nodes.before_source_node(source)?;
        let node = self.nodes.get(id)?;
        match node.links {
            worth_query_consumer_values::BoxNodeLinks::Leaf { source: actual }
                if actual == source =>
            {
                Ok(node.bounds)
            }
            _ => Err(invalid()),
        }
    }
}
