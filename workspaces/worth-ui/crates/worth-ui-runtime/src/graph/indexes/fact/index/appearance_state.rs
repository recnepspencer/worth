use worth_ui_dsl::UiAppearanceStateAxis;

use super::super::{UiGraphFactIndexBasis, UiGraphFactLookupDenial};
use crate::graph::UiGraphNodeIdentity;

impl super::UiGraphConsumedFactIndex {
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
        Ok(
            crate::runtime::appearance::UiAppearanceStateConsumerSelection::new(
                self.basis,
                axis,
                self.appearance_consumers
                    .state_consumers(axis)
                    .iter()
                    .filter(|consumer| consumer.graph_node() == graph_node)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
        )
    }
}
