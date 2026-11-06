//! Original performed-source provenance survives its real output completion.
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryFulfilledRequiredRoot,
};

/// Actual performed source and its exact retained occurrence, moved by the
/// completed output owner. This supplies no current permission or adoption proof.
/// It does not identify the root producer's later publication as the source action.
/// ```compile_fail,E0451
/// use worth_query_publication::facade::application_entry::WorthQueryPerformedProgramOutputSource;
/// use worth_query_execution::facade::primary_graph::{WorthQueryApplicationCommitReceipt, WorthQueryFulfilledRequiredRoot};
/// fn forge(receipt: WorthQueryApplicationCommitReceipt, fulfillment: WorthQueryFulfilledRequiredRoot) -> WorthQueryPerformedProgramOutputSource {
///     WorthQueryPerformedProgramOutputSource { receipt, fulfillment }
/// }
/// ```
/// Borrow the original receipt from an actual settled output tree.
/// ```
/// use worth_query_publication::facade::application_entry::WorthQueryApplicationProgramOutputSettlement;
/// fn source<Query>(settlement: &WorthQueryApplicationProgramOutputSettlement<Query>) {
///     if let Some(original) = settlement.performed_source() {
///         let _receipt = original.receipt();
///     }
/// }
/// ```
pub struct WorthQueryPerformedProgramOutputSource {
    receipt: WorthQueryApplicationCommitReceipt,
    fulfillment: WorthQueryFulfilledRequiredRoot,
}

impl WorthQueryPerformedProgramOutputSource {
    pub(in crate::application_entry::mutation) fn capture(
        receipt: WorthQueryApplicationCommitReceipt,
        fulfillment: WorthQueryFulfilledRequiredRoot,
    ) -> Self {
        Self {
            receipt,
            fulfillment,
        }
    }

    /// Original action receipt. Inspection still requires fresh current access.
    pub fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
    /// Retrospective native root fulfillment; this grants no current permission.
    pub fn root_fulfillment(&self) -> &WorthQueryFulfilledRequiredRoot {
        &self.fulfillment
    }
}
