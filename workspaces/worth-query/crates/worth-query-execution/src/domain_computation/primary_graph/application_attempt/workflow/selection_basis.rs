//! The parts of one selected product operation that a workflow preparation reads.
//!
//! A workflow attempt prepares against the runtime that composes it, the product
//! occurrence its selection admitted and the program inspection bound at that
//! selection. `product_operation` owns the selection and lends exactly these parts
//! for one preparation; the attempt never names the selection itself. The basis
//! grants nothing the selection did not already hold.

/// One selection's parts, borrowed for a single workflow preparation.
pub(in crate::domain_computation::primary_graph) struct WorkflowSelectionBasis<'selection, Schema> {
    application: &'selection dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    product: &'selection crate::basis::WorthQueryProductBranchLease,
    selected_program: Result<
        crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspection,
        crate::domain_computation::primary_graph::vocabulary::product_activation::inspection::WorthQuerySelectedProgramInspectionDenial,
    >,
}

impl<'selection, Schema> WorkflowSelectionBasis<'selection, Schema> {
    pub(in crate::domain_computation::primary_graph) fn new(
        application: &'selection dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
        product: &'selection crate::basis::WorthQueryProductBranchLease,
        selected_program: Result<
            crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspection,
            crate::domain_computation::primary_graph::vocabulary::product_activation::inspection::WorthQuerySelectedProgramInspectionDenial,
        >,
    ) -> Self {
        Self {
            application,
            product,
            selected_program,
        }
    }

    /// The runtime the selection was admitted on, as the attempt's host.
    pub(in crate::domain_computation::primary_graph) fn application(
        &self,
    ) -> &'selection dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>{
        self.application
    }

    /// The exact product occurrence the selection admitted.
    pub(in crate::domain_computation::primary_graph) fn product(
        &self,
    ) -> &'selection crate::basis::WorthQueryProductBranchLease {
        self.product
    }

    /// The rostered program the selection carried, as it was inspected at selection.
    pub(in crate::domain_computation::primary_graph) fn inspect_selected_program(
        &self,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspection,
        crate::domain_computation::primary_graph::vocabulary::product_activation::inspection::WorthQuerySelectedProgramInspectionDenial,
    >{
        self.selected_program.clone()
    }
}
