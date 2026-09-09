#![allow(
    dead_code,
    reason = "Gate 1 retains graph-owned appearance state selection for later consumers"
)]

use worth_ui_dsl::UiAppearanceStateAxis;

use super::super::{UiGraphFactIndexBasis, UiGraphFactLookupDenial};
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
    pub(crate) fn select_appearance_state_consumers(
        &self,
        requested_basis: UiGraphFactIndexBasis,
        axis: UiAppearanceStateAxis,
        graph_node: UiGraphNodeIdentity,
    ) -> Result<
        crate::runtime::appearance::UiAppearanceStateConsumerSelection,
        UiGraphFactLookupDenial,
    > {
        if requested_basis != self.basis {
            return Err(UiGraphFactLookupDenial::BasisMismatch {
                index_basis: self.basis,
                requested_basis,
            });
        }
        let consumers = self.appearance_consumers.state_consumers(axis);
        let start = consumers.partition_point(|consumer| consumer.graph_node() < graph_node);
        let end = consumers.partition_point(|consumer| consumer.graph_node() <= graph_node);
        Ok(
            crate::runtime::appearance::UiAppearanceStateConsumerSelection::new(
                self.basis,
                axis,
                consumers[start..end].into(),
            ),
        )
    }
}
