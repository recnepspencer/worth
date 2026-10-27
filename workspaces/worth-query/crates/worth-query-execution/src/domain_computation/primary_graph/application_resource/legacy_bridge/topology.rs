use super::WorthQueryOwnedAsyncRuntimeServices;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOwnedAsyncRuntimeTopology {
    signal_graph_instance: u64,
    installed_conditional_nodes: usize,
    installed_async_declarations: usize,
    active_signal_nodes: usize,
}

impl WorthQueryOwnedAsyncRuntimeTopology {
    pub const fn signal_graph_instance(self) -> u64 {
        self.signal_graph_instance
    }
    pub const fn installed_conditional_nodes(self) -> usize {
        self.installed_conditional_nodes
    }
    pub const fn installed_async_declarations(self) -> usize {
        self.installed_async_declarations
    }
    pub const fn active_signal_nodes(self) -> usize {
        self.active_signal_nodes
    }
}

impl WorthQueryOwnedAsyncRuntimeServices<'_> {
    pub fn topology(
        &self,
        installed_conditional_nodes: usize,
        installed_async_declarations: usize,
    ) -> Option<WorthQueryOwnedAsyncRuntimeTopology> {
        self.bridge
            .owned_signal_active_node_count()
            .ok()
            .map(|active_signal_nodes| WorthQueryOwnedAsyncRuntimeTopology {
                signal_graph_instance: self.bridge.owned_signal_graph_instance_id(),
                installed_conditional_nodes,
                installed_async_declarations,
                active_signal_nodes,
            })
    }
}
