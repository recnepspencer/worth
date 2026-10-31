use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial, product::WorthQueryOwnedProductHead,
};
use worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial;
use worth_query_installation::facade::WorthQueryDocumentHistoryInstallationDenial;

#[derive(Debug)]
pub enum WorthQueryApplicationHistoryDenial {
    RequestTargetMismatch,
    ForeignRuntime,
    Installation(WorthQueryDocumentHistoryInstallationDenial),
    Query(super::super::WorthQueryApplicationRequestQueryDenial),
    Document(WorthQueryDocumentHistoryDenial),
    ExecutionRequest(WorthQueryAdvancementDenial),
}

#[must_use = "retain or close the original head after refused attachment"]
pub struct WorthQueryApplicationHistoryAttachmentFailure<'runtime, S> {
    pub(super) head: WorthQueryOwnedProductHead<'runtime, S>,
    pub(super) denial: WorthQueryApplicationHistoryDenial,
}

impl<'runtime, S> WorthQueryApplicationHistoryAttachmentFailure<'runtime, S> {
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryOwnedProductHead<'runtime, S>,
        WorthQueryApplicationHistoryDenial,
    ) {
        (self.head, self.denial)
    }
    pub fn denial(&self) -> &WorthQueryApplicationHistoryDenial {
        &self.denial
    }
}

impl<S> std::fmt::Debug for WorthQueryApplicationHistoryAttachmentFailure<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationHistoryAttachmentFailure")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}
