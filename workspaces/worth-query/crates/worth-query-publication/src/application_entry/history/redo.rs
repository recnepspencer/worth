//! Bounded completed-child discovery; every hint remains descriptive until NAV admission.
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};

impl<S, H, Payload, Outputs> WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    /// Classify completed children and read a bounded page under fresh metadata admission.
    /// Even Single is a hint: pass its child to redo_to with a current expected selection.
    pub fn redo_choices(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
    ) -> Result<WorthQueryHistoryRedoChoices, WorthQueryApplicationHistoryDenial> {
        self.redo_choices_in(request, limits, after, Ungoverned)
    }

    pub fn redo_choices_governed<C, O, I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
    ) -> Result<WorthQueryHistoryRedoChoices, WorthQueryApplicationHistoryDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.redo_choices_in(
            request,
            limits,
            after,
            Governed {
                capability,
                operation,
                input,
            },
        )
    }

    fn redo_choices_in(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
        authority: impl MetadataAuthority<S, H>,
    ) -> Result<WorthQueryHistoryRedoChoices, WorthQueryApplicationHistoryDenial> {
        DocumentRequestContext::bind(&self.owner, request)?.with_metadata(
            self.access.clone(),
            &self.payload,
            authority,
            |phase, runtime, read| {
                runtime.read_document_history_redo_choices(
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
