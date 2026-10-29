//! Controlled finite operations consumed by the Query-free AVL mechanism.
use super::*;
use worth_query_consumer_values::{
    compare_prefix_keys, prefix_order, PrefixBuildStorage, PrefixOrder, PrefixQueryStorage,
};

pub(super) struct Build<'call, 'source, 'node, 'control> {
    pub sources: &'call WorthQueryManagedIndexSourceRows<'source, Catalog, PrefixIndex>,
    pub nodes: &'call mut WorthQueryManagedIndexNodeWriter<'node, Node>,
    pub control: &'call WorthQueryManagedIndexControl<'control>,
    pub after_first_node: &'call Mutex<Option<Box<dyn FnOnce() + Send>>>,
}
fn invalid() -> Stop {
    Stop::Domain(PrefixIndexDenial::InvalidTopology)
}
fn name(
    sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
    ordinal: usize,
) -> Result<String, Stop> {
    let (_, row) = sources.at(ordinal)?;
    // Actual codec refusals are latched by the framework invocation before this
    // structural adapter refusal can become its final domain result.
    row.field(query::name()).map_err(|_| invalid())
}
impl PrefixBuildStorage for Build<'_, '_, '_, '_> {
    type Id = WorthQueryManagedIndexNodeId;
    type Source = usize;
    type Stop = Stop;
    fn checkpoint(&mut self) -> Result<(), Stop> {
        self.control.checkpoint(1).map_err(Into::into)
    }
    fn compare_sources(&mut self, left: usize, right: usize) -> Result<std::cmp::Ordering, Stop> {
        let a = name(self.sources, left)?;
        let b = name(self.sources, right)?;
        // Canonical source ordinals have the same order as their full-GEN keys.
        let a_key = u64::try_from(left).map_err(|_| invalid())?;
        let b_key = u64::try_from(right).map_err(|_| invalid())?;
        compare_prefix_keys(&a, a_key, &b, b_key, |units| {
            self.control.checkpoint(units).map_err(Into::into)
        })
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
pub(super) struct Query<'call, 'source, 'candidate, 'control> {
    pub sources: &'call WorthQueryManagedIndexSourceRows<'source, Catalog, PrefixIndex>,
    pub nodes: &'call WorthQueryManagedIndexNodes<Node>,
    pub probe: &'call query::Probe,
    pub candidates: &'call mut WorthQueryManagedIndexCandidateWriter<'candidate>,
    pub control: &'call WorthQueryManagedIndexControl<'control>,
}
impl PrefixQueryStorage for Query<'_, '_, '_, '_> {
    type Id = WorthQueryManagedIndexNodeId;
    type Source = usize;
    type Stop = Stop;
    fn checkpoint(&mut self) -> Result<(), Stop> {
        self.control.checkpoint(1).map_err(Into::into)
    }
    fn read(&mut self, id: Self::Id) -> Result<Node, Stop> {
        self.nodes
            .get(id, self.control)
            .copied()
            .map_err(Into::into)
    }
    fn classify_source(&mut self, ordinal: usize) -> Result<PrefixOrder, Stop> {
        let value = name(self.sources, ordinal)?;
        prefix_order(&value, &self.probe.prefix, |units| {
            self.control.checkpoint(units).map_err(Into::into)
        })
    }
    fn emit(&mut self, ordinal: usize) -> Result<(), Stop> {
        let (slot, _) = self.sources.at(ordinal)?;
        self.candidates.emit(slot).map_err(Into::into)
    }
    fn invalid(&self) -> Stop {
        invalid()
    }
}
