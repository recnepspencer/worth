//! Recorded incarnation bounds affect future allocation, never the recovered source image.
use super::SignalGraph;
impl SignalGraph {
    pub(crate) fn authored_node_issuance_bounds(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.arena.nodes.iter().enumerate().map(|(index, slot)| {
            (
                index as u32,
                if slot.is_retired() {
                    u32::MAX
                } else {
                    slot.generation
                },
            )
        })
    }
    pub(crate) fn install_recorded_node_issuance(
        &mut self,
        bounds: worth_execution::ExecutionOrderedMap<u32, u32>,
    ) {
        self.recorded_node_issuance = bounds;
    }
}
