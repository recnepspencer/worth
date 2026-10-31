//! A sealed description of actual document selection, distinct from a node's source.
use super::super::{WorthQueryHistoryNodeId, WorthQueryHistoryStep};
use crate::basis::WorthQueryProductBranchReadIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryHistorySelection {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) step:
        WorthQueryHistoryStep,
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) physical:
        WorthQueryProductBranchReadIdentity,
}

impl<S, H> super::super::WorthQueryDocumentHistory<'_, S, H>
where
    S: worth_query_installation::facade::ApplicationSchema,
    H: worth_query_declaration::facade::application_history::ApplicationHistoryBinding<S>,
{
    pub(in crate::domain_computation::primary_graph) fn current_history_selection(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> WorthQueryHistorySelection {
        let selected = self.selection.lock();
        WorthQueryHistorySelection {
            step: selected.journal.current_step().clone(),
            physical: selected.expected.clone(),
        }
    }
}

impl WorthQueryHistorySelection {
    /// The selected authored occurrence. Its recorded source remains immutable.
    pub fn step(&self) -> &WorthQueryHistoryStep {
        &self.step
    }
    /// The actual current writable context; navigation creates a new endpoint.
    pub fn physical_endpoint(&self) -> &WorthQueryProductBranchReadIdentity {
        &self.physical
    }
}

/// Description of the original successful navigation, including on a later retry.
/// Read the document again to observe its current selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryHistoryNavigationCompletion {
    pub(super) selection: WorthQueryHistorySelection,
    pub(super) requested: WorthQueryHistoryNodeId,
}

impl WorthQueryHistoryNavigationCompletion {
    pub fn selection(&self) -> &WorthQueryHistorySelection {
        &self.selection
    }
    pub fn requested_node(&self) -> WorthQueryHistoryNodeId {
        self.requested
    }
}
