//! Explicitly refresh physical context after bounded, proven outside-model writes.
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
mod pause;

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
    /// Requires a declared `ScopeEntityRecord` model domain and fresh metadata
    /// authority. Success preserves the current step, endpoints and cursors.
    pub fn refresh_context(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        limits: WorthQueryHistoryContextRefreshLimits,
    ) -> Result<(), WorthQueryApplicationHistoryDenial> {
        self.refresh_context_in(request, limits, Ungoverned)
    }

    pub fn refresh_context_governed<C, O, I>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
        limits: WorthQueryHistoryContextRefreshLimits,
    ) -> Result<(), WorthQueryApplicationHistoryDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.refresh_context_in(request, limits, Governed { capability, operation, input })
    }

    fn refresh_context_in(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        limits: WorthQueryHistoryContextRefreshLimits,
        authority: impl MetadataAuthority<S, H>,
    ) -> Result<(), WorthQueryApplicationHistoryDenial> {
        DocumentRequestContext::bind(&self.owner, request)?.with_metadata(
            self.access.clone(),
            &self.payload,
            authority,
            |phase, runtime, read| {
                runtime.refresh_document_history_context(
                    phase,
                    &program_publication_access(),
                    &mut self.owner,
                    read,
                    limits,
                )
            },
        )
    }
}
