//! Fresh metadata decorates the original READ; output custody never enters this read.
use super::*;
impl<S, H, Payload, Outputs> WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn history_retention_status(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial> {
        self.history_retention_status_in(request, Ungoverned)
    }

    pub fn history_retention_status_governed<C, O, I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.history_retention_status_in(request, Governed { capability, operation, input })
    }

    fn history_retention_status_in(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        authority: impl MetadataAuthority<S, H>,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial> {
        DocumentRequestContext::bind(&self.owner, request)?.with_metadata(
            self.access.clone(),
            &self.payload,
            authority,
            |phase, runtime, read| {
                runtime.read_document_history_retention(
                    phase,
                    &program_publication_access(),
                    &self.owner,
                    read,
                )
            },
        )
    }
}
