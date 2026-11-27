//! Bounded journal pages always use fresh metadata authority and the original model READ.
//! A caller cannot rewrite a sealed continuation position.
//! ```compile_fail,E0451
//! use worth_query_publication::facade::application_entry::WorthQueryHistoryPageCursor;
//! fn rewind(cursor: WorthQueryHistoryPageCursor) -> WorthQueryHistoryPageCursor {
//!     WorthQueryHistoryPageCursor { after: 0, ..cursor }
//! }
//! ```
//! Keep an issued cursor for a fresh admitted retry instead.
//! ```
//! use worth_query_publication::facade::application_entry::WorthQueryHistoryPageCursor;
//! fn keep(cursor: &WorthQueryHistoryPageCursor) -> WorthQueryHistoryPageCursor { *cursor }
//! ```
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
where
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn history_page(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
    ) -> Result<WorthQueryHistoryPage, WorthQueryApplicationHistoryDenial> {
        self.history_page_in(request, limits, after, Ungoverned)
    }
    pub fn history_page_governed<C, O, I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
    ) -> Result<WorthQueryHistoryPage, WorthQueryApplicationHistoryDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.history_page_in(request, limits, after, Governed { capability, operation, input })
    }
    fn history_page_in(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
        authority: impl MetadataAuthority<S, H>,
    ) -> Result<WorthQueryHistoryPage, WorthQueryApplicationHistoryDenial> {
        DocumentRequestContext::bind(&self.owner, request)?.with_metadata(
            self.access.clone(),
            &self.payload,
            authority,
            |phase, runtime, read| {
                runtime.read_document_history_page(
                    phase,
                    &program_publication_access(),
                    &self.owner,
                    read,
                    limits,
                    after,
                )
            },
        )
    }
}
