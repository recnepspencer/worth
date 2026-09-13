use worth_ui_dsl::UiAppearanceStateAxis;

use crate::graph::UiGraphNodeIdentity;

impl super::UiGraphConsumedFactIndex {
    pub(crate) fn has_appearance_attachment(&self, node: UiGraphNodeIdentity) -> bool {
        self.appearance_consumers.has_attached_node(node)
    }

    pub(crate) fn consumes_appearance_state(
        &self,
        axis: UiAppearanceStateAxis,
        graph_node: UiGraphNodeIdentity,
    ) -> bool {
        self.appearance_consumers
            .state_consumer_nodes(axis)
            .binary_search(&graph_node)
            .is_ok()
    }
}
