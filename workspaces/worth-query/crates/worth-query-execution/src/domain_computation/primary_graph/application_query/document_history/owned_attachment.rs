//! A document attached through its installed program runtime keeps that runtime alive.
use super::{Denial, Query, Rows, WorthQueryDocumentHistoryRead};
use crate::domain_computation::primary_graph::application_installation::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::primary_graph::product_operation::{
    SharedPrimaryRuntime, WorthQueryRuntimeCustody,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationProjection, WorthQueryDocumentHistory,
    WorthQueryDocumentHistoryAttachmentFailure, WorthQueryOwnedProductHead,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding,
    application_program::ApplicationProgramDefinition, application_schema::ApplicationSchema,
};

impl<S, P> SharedPrimaryRuntime<S> for WorthQueryProgramApplicationRuntime<S, P>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
{
    fn runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<S> {
        self.backing.runtime()
    }
}

impl<S, P> WorthQueryProgramApplicationRuntime<S, P>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
{
    /// Attach a document that holds this installed runtime alive until it closes.
    ///
    /// The attachment itself is the ordinary one. A head of another runtime is
    /// refused before any read, and every refusal returns the head exactly as
    /// the caller supplied it, still borrowing its runtime.
    pub fn attach_owned_document_history<'head, H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        head: WorthQueryOwnedProductHead<'head, S>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<
        WorthQueryDocumentHistory<'static, S, H>,
        WorthQueryDocumentHistoryAttachmentFailure<'head, S>,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        if !head.belongs_to(self.runtime()) {
            return Err(WorthQueryDocumentHistoryAttachmentFailure {
                head,
                denial: Denial::ForeignRuntime,
            });
        }
        let shared = WorthQueryRuntimeCustody::Shared(std::sync::Arc::new(self.clone()));
        let (head, borrowed) = head.exchange_custody(shared);
        self.runtime()
            .attach_document_history(phase, access, head, read)
            .map_err(|failure| {
                let (head, denial) = failure.into_parts();
                let (head, _shared) = head.exchange_custody(borrowed);
                WorthQueryDocumentHistoryAttachmentFailure { head, denial }
            })
    }
}
