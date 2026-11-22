use super::WorthQueryOwnedProductHead;

pub use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::document_history::WorthQueryDocumentHistoryDenial;

/// No document was attached. The original live head and actual read denial remain owned.
#[must_use = "retain or close the original head after refused attachment"]
pub struct WorthQueryDocumentHistoryAttachmentFailure<'runtime, S> {
    pub(in crate::domain_computation::primary_graph) head: WorthQueryOwnedProductHead<'runtime, S>,
    pub(in crate::domain_computation::primary_graph) denial: WorthQueryDocumentHistoryDenial,
}

impl<'runtime, S> WorthQueryDocumentHistoryAttachmentFailure<'runtime, S> {
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryOwnedProductHead<'runtime, S>,
        WorthQueryDocumentHistoryDenial,
    ) {
        (self.head, self.denial)
    }
    pub fn denial(&self) -> &WorthQueryDocumentHistoryDenial {
        &self.denial
    }
}

impl<S> std::fmt::Debug for WorthQueryDocumentHistoryAttachmentFailure<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryDocumentHistoryAttachmentFailure")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}
#[cfg(test)]
#[path = "denial_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
