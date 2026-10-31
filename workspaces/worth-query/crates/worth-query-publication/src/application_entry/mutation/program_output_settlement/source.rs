//! Original performed-source provenance survives its real output completion.
use std::sync::Arc;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationReadObservation,
};

/// Actual performed source and its exact retained occurrence, moved by the
/// completed output owner. This supplies no current permission or adoption proof.
/// It does not identify the root producer's later publication as the source action.
/// ```compile_fail,E0451
/// use worth_query_publication::facade::application_entry::WorthQueryPerformedProgramOutputSource;
/// use worth_query_execution::facade::primary_graph::{WorthQueryApplicationCommitReceipt, WorthQueryApplicationReadObservation};
/// fn forge(receipt: WorthQueryApplicationCommitReceipt, source: std::sync::Arc<WorthQueryApplicationReadObservation>) -> WorthQueryPerformedProgramOutputSource {
///     WorthQueryPerformedProgramOutputSource { receipt, _source: source }
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
    // No getter: a descriptive provenance loan cannot escape as a selector.
    _source: Arc<WorthQueryApplicationReadObservation>,
}

impl WorthQueryPerformedProgramOutputSource {
    pub(in crate::application_entry::mutation) fn capture(
        receipt: WorthQueryApplicationCommitReceipt,
        source: &crate::application_entry::WorthQueryApplicationReadObservation,
    ) -> Self {
        Self {
            receipt,
            _source: Arc::clone(&source.retained),
        }
    }

    /// Original action receipt. Inspection still requires fresh current access.
    pub fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
}
