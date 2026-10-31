//! The same spatial insertion runs over original-paid cold node backing.
use super::*;
use worth_query_consumer_values::BoxEditStorage;
#[cfg(test)]
use worth_query_consumer_values::BoxNodeLinks;

pub(super) struct Storage<'call, 'node, 'control> {
    pub nodes: &'call mut WorthQueryManagedIndexNodeWriter<'node, Node>,
    pub control: &'call WorthQueryManagedIndexControl<'control>,
    pub calls: &'call Calls,
}
impl BoxEditStorage for Storage<'_, '_, '_> {
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
        let id = self.nodes.push(node)?;
        if self.nodes.len() == 1 {
            let observer = self.calls.after_first_node.lock().unwrap().take();
            if let Some(observe) = observer {
                observe();
            }
        }
        Ok(id)
    }
    fn replace(&mut self, id: Self::Id, node: Node) -> Result<(), Stop> {
        self.nodes.replace(id, node).map_err(Into::into)
    }
    fn remove(&mut self, id: Self::Id) -> Result<(), Stop> {
        self.nodes.remove(id).map_err(Into::into)
    }
    fn source_bounds(&mut self, source: Self::Source) -> Result<Box2, Stop> {
        let id = self.nodes.before_source_node(source)?;
        Ok(self.nodes.get(id)?.bounds)
    }
    fn invalid(&self) -> Stop {
        Stop::Domain(BoxIndexDenial::InvalidTopology)
    }
}

#[cfg(test)]
pub(super) fn corrupt_leaf(
    nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
    root: Option<WorthQueryManagedIndexNodeId>,
) -> Result<(), Stop> {
    let Some(mut id) = root else { return Ok(()) };
    for _ in 0..nodes.len() {
        let mut node = nodes.get(id)?;
        match node.links {
            BoxNodeLinks::Branch { left, .. } => id = left,
            BoxNodeLinks::Leaf { .. } => {
                node.bounds =
                    Box2::new(Coordinates::new(10_000, 10_000, 10_001, 10_001).raw()).unwrap();
                return nodes.replace(id, node).map_err(Into::into);
            }
        }
    }
    Err(Stop::Domain(BoxIndexDenial::InvalidTopology))
}
