//! Same-cell staging preserves liveness and compares only authored storage roots.
use super::SignalGraph;
impl SignalGraph {
    pub(crate) fn preserves_conditional_definitions_of(&self, previous: &Self) -> bool {
        for node in previous.live_node_ids() {
            let before = &previous.arena.definitions[node.index() as usize];
            if let Some(source) = &before.conditional_definition {
                if !self.is_alive(node) {
                    return false;
                }
                let after = &self.arena.definitions[node.index() as usize];
                if !after
                    .conditional_definition
                    .as_ref()
                    .is_some_and(|current| std::sync::Arc::ptr_eq(source, current))
                    || before.eval_config != after.eval_config
                    || before.tombstoned != after.tombstoned
                    || before.declared_dependencies != after.declared_dependencies
                {
                    return false;
                }
            }
        }
        self.live_node_ids().into_iter().all(|node| {
            self.arena.definitions[node.index() as usize]
                .conditional_definition
                .is_none()
                || (previous.is_alive(node)
                    && previous.arena.definitions[node.index() as usize]
                        .conditional_definition
                        .is_some())
        })
    }
    pub(crate) fn fork_durable_successor(&mut self) -> Box<Self> {
        let lifecycle = self.lifecycle_token.clone();
        let (mut successor, _) = self.fork_conditional_successor();
        successor.lifecycle_token = lifecycle;
        Box::new(successor)
    }
    pub(crate) fn shares_authored_storage_with(&self, other: &Self) -> bool {
        self.arena.nodes.shares_storage_with(&other.arena.nodes)
            && self
                .arena
                .definitions
                .shares_storage_with(&other.arena.definitions)
            && self
                .arena
                .free_list
                .shares_storage_with(&other.arena.free_list)
    }
}
