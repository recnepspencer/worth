//! Navigation selects an existing occurrence without rewriting or appending its source.
use super::*;

impl<S> HistoryJournal<'_, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn prepare_navigation(
        &self,
        node: WorthQueryHistoryNodeId,
    ) -> Result<u64, Denial> {
        if self.pending.is_some() {
            return Err(Denial::AttemptInFlight);
        }
        if !self.owns_node(node) {
            return Err(Denial::ForeignHistoryNode);
        }
        if self.step(node).is_none() {
            return Err(Denial::HistoryNodeUnavailable);
        }
        self.view_generation
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn complete_navigation(
        &mut self,
        node: WorthQueryHistoryNodeId,
        view_after: u64,
    ) {
        // The caller holds the actual selection guard and rechecked its original
        // physical selection. Target and generation were admitted before effects.
        debug_assert!(self.step(node).is_some());
        debug_assert_eq!(self.view_generation.checked_add(1), Some(view_after));
        self.current = node;
        self.view_generation = view_after;
    }
}
