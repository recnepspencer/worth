use crate::data::graph::runtime::graph::NodeArena;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;

impl SignalGraph {
    pub fn is_alive(&self, id: NodeId) -> bool {
        let idx = id.index() as usize;
        if idx >= self.arena.nodes.len() {
            return false;
        }
        let slot = &self.arena.nodes[idx];
        slot.generation == id.generation() && slot.is_occupied()
    }

    pub fn active_node_count(&self) -> usize {
        self.arena.active_nodes as usize
    }

    pub fn arena_capacity(&self) -> usize {
        self.arena.nodes.len()
    }

    pub(crate) fn live_node_id_at(&self, index: usize) -> Option<NodeId> {
        let slot = self.arena.nodes.get(index)?;
        if !slot.is_occupied() {
            return None;
        }
        Some(NodeId::new(index as u32, slot.generation))
    }
}

impl NodeArena {
    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }
}

impl SignalGraph {
    pub(crate) fn authored_conditional_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.arena
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                (slot.is_occupied()
                    && self.arena.definitions[index]
                        .conditional_definition
                        .is_some())
                .then_some(NodeId::new(index as u32, slot.generation))
            })
    }

    pub(crate) fn live_node_ids(&self) -> Vec<NodeId> {
        self.arena
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                slot.is_occupied()
                    .then_some(NodeId::new(index as u32, slot.generation))
            })
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn storage_counts(&self) -> ((usize, usize), (usize, usize), usize) {
        (
            self.topology.dependency_edges.storage_counts(),
            self.topology.subscriber_edges.storage_counts(),
            self.topology.dependency_snapshots.snapshot_count(),
        )
    }
}
