//! Controlled finite operations consumed by the Query-free AVL mechanism.
use super::*;
use worth_query_consumer_values::{compare_prefix_keys, PrefixBuildStorage};

pub(super) struct Build<'call, 'source, 'node, 'control> {
    pub sources: &'call WorthQueryManagedIndexSourceRows<'source, Catalog, PrefixIndex>,
    pub nodes: &'call mut WorthQueryManagedIndexNodeWriter<'node, Node>,
    pub control: &'call WorthQueryManagedIndexControl<'control>,
    pub after_first_node: &'call Mutex<Option<Box<dyn FnOnce() + Send>>>,
}
fn invalid() -> Stop {
    Stop::Domain(PrefixIndexDenial::InvalidTopology)
}
impl PrefixBuildStorage for Build<'_, '_, '_, '_> {
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
    ) -> Result<std::cmp::Ordering, Stop> {
        let a = self.sources.get(left)?;
        let b = self.sources.get(right)?;
        a.with_text(query::name(), |a| {
            b.with_text(query::name(), |b| {
                compare_prefix_keys(a, 0, b, 0, |units| {
                    self.control.checkpoint(units).map_err(Into::into)
                })
                .map(|order| order.then_with(|| left.cmp(&right)))
            })
            .map_err(text_denial)?
        })
        .map_err(text_denial)?
    }
    fn append(&mut self, node: Node) -> Result<Self::Id, Stop> {
        let id = self.nodes.push(node)?;
        if self.nodes.len() == 1 {
            let observe = self.after_first_node.lock().unwrap().take();
            if let Some(observe) = observe {
                observe();
            }
        }
        Ok(id)
    }
    fn read(&mut self, id: Self::Id) -> Result<Node, Stop> {
        self.nodes.get(id).map_err(Into::into)
    }
    fn replace(&mut self, id: Self::Id, node: Node) -> Result<(), Stop> {
        self.nodes.replace(id, node).map_err(Into::into)
    }
    fn remove(&mut self, id: Self::Id) -> Result<(), Stop> {
        self.nodes.remove(id).map_err(Into::into)
    }
    fn invalid(&self) -> Stop {
        invalid()
    }
}
