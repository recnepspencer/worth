//! Workflow preparations against a selected product.
//!
//! Each workflow publication adapter takes the caller's selected product
//! operation and lends the attempt only the parts it prepares on, a
//! `WorkflowSelectionBasis`: the runtime as the attempt's host, the admitted
//! product occurrence and the program inspected at selection. The preparation
//! logic, its read sets and its sealed authority stay in `application_attempt`;
//! the entries that hold a selection live here, one child per program.
use super::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::application_attempt::WorkflowSelectionBasis;
use worth_query_installation::facade::ApplicationSchema;

mod transition;
pub use transition::WorthQueryWorkflowAdvanceAdapter;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// The parts of this selection that one workflow preparation reads.
    pub(in crate::domain_computation::primary_graph) fn workflow_selection_basis(
        &self,
    ) -> WorkflowSelectionBasis<'_, Schema> {
        WorkflowSelectionBasis::new(
            self.application().application_attempt_host(),
            self.product(),
            self.inspect_selected_program(),
        )
    }
}
