use crate::data::error::SignalError;
use crate::data::graph::signal_graph::{stale_error, SignalGraph};
use crate::data::graph::storage::Slot;
use crate::data::handle::NodeId;
use crate::data::node::{NodeDefinitionData, NodeEntry, NodeWarmData};

const NODE_ARENA_RESERVE_CHUNK: usize = 1024;

impl SignalGraph {
    pub(in crate::data::graph) fn allocate_node(&mut self, entry: NodeEntry) -> NodeId {
        let (definition, hot, warm, cold) = entry.into_storage_parts();
        while let Some(index) = self.arena.free_list.pop_back() {
            if index as usize >= self.arena.nodes.len() {
                continue;
            }
            self.arena.free_slots.clear(index as usize);
            let slot = &mut self.arena.nodes[index as usize];
            if slot.is_retired() {
                continue;
            }
            self.arena.definitions[index as usize] = definition.clone();
            self.arena.hot[index as usize] = Some(hot.clone());
            self.arena.warm[index as usize] = warm.clone();
            self.arena.cold[index as usize] = cold.clone();
            let generation = slot.occupy();
            self.arena.active_nodes += 1;
            let node = NodeId::new(index, generation);
            self.record_branch_mutation_introduced(node);
            return node;
        }

        let index = self.arena.nodes.len() as u32;
        if self.arena.nodes.exclusive_capacity() == Some(self.arena.nodes.len()) {
            self.reserve_node_capacity(NODE_ARENA_RESERVE_CHUNK);
        }
        let mut slot = Slot::vacant();
        let generation = slot.occupy();
        self.arena.nodes.push_back(slot);
        self.arena.definitions.push_back(definition);
        self.arena.hot.push_back(Some(hot));
        self.arena.warm.push_back(warm);
        self.arena.cold.push_back(cold);
        self.arena.active_nodes += 1;
        let node = NodeId::new(index, generation);
        self.record_branch_mutation_introduced(node);
        node
    }

    pub(crate) fn rollback_created_nodes(&mut self, created_nodes: &[NodeId]) {
        for node in created_nodes {
            self.conditional_dependency_versions.remove(node);
        }
        let mut indices = created_nodes
            .iter()
            .map(|node| node.index() as usize)
            .collect::<Vec<_>>();
        indices.sort_unstable();
        indices.dedup();
        self.observation
            .telemetry
            .storage
            .rolled_back_created_node_count += indices.len() as u64;

        let mut newly_freed = Vec::with_capacity(indices.len());
        for index in indices.iter().rev().copied() {
            let Some(slot) = self.arena.nodes.get_mut(index) else {
                continue;
            };
            if slot.is_occupied() {
                slot.vacate();
                self.arena.definitions[index] = NodeDefinitionData::default();
                self.arena.hot[index] = None;
                self.arena.warm[index] = NodeWarmData::default();
                self.arena.cold[index] = None;
                self.arena.active_nodes = self.arena.active_nodes.saturating_sub(1);
                if !slot.is_retired() && !self.arena.free_slots.contains(index) {
                    self.arena.free_slots.mark(index);
                    newly_freed.push(index);
                }
            }
        }

        newly_freed.sort_unstable();
        loop {
            let Some(last_index) = self.arena.nodes.len().checked_sub(1) else {
                break;
            };
            if newly_freed.binary_search(&last_index).is_err()
                || self.arena.nodes[last_index].is_occupied()
            {
                break;
            }
            self.arena.free_slots.clear(last_index);
            self.arena.nodes.pop_back();
            self.arena.definitions.pop_back();
            self.arena.hot.pop_back();
            self.arena.warm.pop_back();
            self.arena.cold.pop_back();
        }
        for index in newly_freed {
            if index < self.arena.nodes.len() && self.arena.free_slots.contains(index) {
                self.arena.free_list.push_back(index as u32);
            }
        }
    }

    pub(crate) fn node_allocator_state(&self) -> u32 {
        self.arena.nodes.len() as u32
    }

    pub(crate) fn synchronize_node_allocator(&mut self, next_node_index: u32) {
        if self.arena.nodes.len() as u32 >= next_node_index {
            return;
        }
        let missing = next_node_index as usize - self.arena.nodes.len();
        self.reserve_node_capacity(missing);
        for _ in 0..missing {
            self.arena.nodes.push_back(Slot::retired_placeholder());
            self.arena
                .definitions
                .push_back(NodeDefinitionData::default());
            self.arena.hot.push_back(None);
            self.arena.warm.push_back(NodeWarmData::default());
            self.arena.cold.push_back(None);
        }
    }

    pub(crate) fn reserve_node_capacity(&mut self, additional: usize) {
        self.arena.nodes.reserve_exclusive(additional);
        self.arena.definitions.reserve_exclusive(additional);
        self.arena.hot.reserve_exclusive(additional);
        self.arena.warm.reserve_exclusive(additional);
        self.arena.cold.reserve_exclusive(additional);
    }

    pub(in crate::data::graph) fn validate_handle(&self, id: NodeId) -> Result<(), SignalError> {
        let idx = id.index() as usize;
        if idx >= self.arena.nodes.len() {
            return Err(stale_error(id, id.generation()));
        }
        let slot = &self.arena.nodes[idx];
        if slot.generation != id.generation() || !slot.is_occupied() {
            return Err(stale_error(id, slot.generation));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn free_list_snapshot(&self) -> Vec<u32> {
        self.arena.free_list.iter().copied().collect()
    }

    #[cfg(test)]
    pub(crate) fn force_slot_generation_for_test(
        &mut self,
        index: u32,
        generation: u32,
    ) -> Result<(), SignalError> {
        let slot = self
            .arena
            .nodes
            .get_mut(index as usize)
            .ok_or_else(|| SignalError::invalid_input(format!("unknown slot `{index}`")))?;
        slot.generation = generation;
        slot.retired = false;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn is_slot_retired_for_test(&self, index: u32) -> Result<bool, SignalError> {
        let slot = self
            .arena
            .nodes
            .get(index as usize)
            .ok_or_else(|| SignalError::invalid_input(format!("unknown slot `{index}`")))?;
        Ok(slot.is_retired())
    }
}
