//! Reverse authored adjacency survives changes to the discovered read set.
use crate::data::dependency::CanonicalDependencies;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;

impl SignalGraph {
    /// Reconstruction cost only; ordinary declaration changes update the index locally.
    pub(in crate::data::graph) fn rebuild_declared_dependents(&mut self) {
        self.topology.declared_dependents = Default::default();
        for index in 0..self.arena.nodes.len() {
            let slot = &self.arena.nodes[index];
            if !slot.is_occupied() {
                continue;
            }
            let node = NodeId::new(index as u32, slot.generation);
            let edges = self.arena.definitions[index].declared_dependencies.take();
            if let Some(edges) = edges {
                self.install_declared_dependencies(node, edges);
            }
        }
    }

    pub(in crate::data::graph) fn install_declared_dependencies(
        &mut self,
        node: NodeId,
        desired: CanonicalDependencies,
    ) {
        let previous = self.arena.definitions[node.index() as usize]
            .declared_dependencies
            .take();
        if let Some(previous) = previous {
            for edge in previous.as_slice() {
                let source = edge.source();
                if let Some(mut consumers) = self.topology.declared_dependents.get(&source).cloned()
                {
                    consumers.remove(&node);
                    if consumers.is_empty() {
                        self.topology.declared_dependents.remove(&source);
                    } else {
                        self.topology.declared_dependents.insert(source, consumers);
                    }
                }
            }
        }
        for edge in desired.as_slice() {
            let source = edge.source();
            let mut consumers = self
                .topology
                .declared_dependents
                .get(&source)
                .cloned()
                .unwrap_or_default();
            consumers.insert(node);
            self.topology.declared_dependents.insert(source, consumers);
        }
        self.arena.definitions[node.index() as usize].declared_dependencies =
            (!desired.is_empty()).then_some(desired);
    }

    pub(in crate::data::graph) fn retire_declared_adjacency(&mut self, retired: NodeId) {
        let consumers = self
            .topology
            .declared_dependents
            .get(&retired)
            .cloned()
            .unwrap_or_default();
        for consumer in consumers {
            let retained = CanonicalDependencies::new(
                self.arena.definitions[consumer.index() as usize]
                    .declared_dependencies
                    .as_ref()
                    .into_iter()
                    .flat_map(|edges| edges.as_slice())
                    .filter(|edge| edge.source() != retired)
                    .cloned(),
            );
            self.install_declared_dependencies(consumer, retained);
        }
        self.install_declared_dependencies(retired, CanonicalDependencies::new([]));
    }
}
