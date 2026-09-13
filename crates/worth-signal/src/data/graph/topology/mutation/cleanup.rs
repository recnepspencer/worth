#[cfg(any(test, debug_assertions))]
use crate::data::error::SignalError;
#[cfg(any(test, debug_assertions))]
use crate::data::handle::NodeId;

use crate::data::graph::signal_graph::SignalGraph;

impl SignalGraph {
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn assert_bidirectional_consistency(&self) -> Result<(), SignalError> {
        for (index, slot) in self.arena.nodes.iter().enumerate() {
            if !slot.is_occupied() {
                continue;
            }
            let entry = self.get_entry(NodeId::new(index as u32, slot.generation))?;
            if entry.is_tombstoned() {
                continue;
            }
            let node = NodeId::new(index as u32, slot.generation);

            for dependency in self
                .topology
                .dependency_edges
                .get(entry.get_dependencies_id())
            {
                if !self.is_alive(dependency.source()) {
                    continue;
                }
                if !self
                    .topology
                    .subscriber_edges
                    .get(self.get_entry(dependency.source())?.get_subscribers_id())
                    .contains(&node)
                {
                    return Err(SignalError::internal(format!(
                        "topology inconsistency: missing subscriber edge {} -> {}",
                        dependency.source(),
                        node
                    )));
                }
            }

            for &subscriber in self
                .topology
                .subscriber_edges
                .get(entry.get_subscribers_id())
            {
                if !self.is_alive(subscriber) {
                    continue;
                }
                if !self
                    .topology
                    .dependency_edges
                    .get(self.get_entry(subscriber)?.get_dependencies_id())
                    .iter()
                    .any(|dependency| dependency.source() == node)
                {
                    return Err(SignalError::internal(format!(
                        "topology inconsistency: missing dependency edge {} -> {}",
                        node, subscriber
                    )));
                }
            }
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn debug_assert_bidirectional_consistency(&self) {
        #[cfg(debug_assertions)]
        if topology_debug_asserts_enabled() {
            self.assert_bidirectional_consistency()
                .expect("signal topology should remain bidirectionally consistent");
        }
    }
}

#[cfg(debug_assertions)]
fn topology_debug_asserts_enabled() -> bool {
    std::env::var_os("WORTH_SIGNAL_SKIP_TOPOLOGY_DEBUG_ASSERTS").is_none()
}
