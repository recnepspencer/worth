//! A bookmark operation supplies fresh metadata and its independent role authority.
use super::*;
impl<S, H, Payload, Outputs> WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn set_history_bookmark<N: ApplicationHistoryBookmarkBinding<S, H>>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        node: WorthQueryHistoryNodeId,
        pin: bool,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial> {
        self.set_history_bookmark_in::<N>(request, node, pin, Ungoverned)
    }

    pub fn set_history_bookmark_governed<N, C, O, I>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        node: WorthQueryHistoryNodeId,
        pin: bool,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial>
    where
        N: ApplicationHistoryBookmarkBinding<S, H>,
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.set_history_bookmark_in::<N>(request, node, pin, Governed { capability, operation, input })
    }

    fn set_history_bookmark_in<N: ApplicationHistoryBookmarkBinding<S, H>>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        node: WorthQueryHistoryNodeId,
        pin: bool,
        authority: impl MetadataAuthority<S, H>,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial> {
        DocumentRequestContext::bind(&self.owner, request)?.with_metadata(
            self.access.clone(),
            &self.payload,
            authority,
            |phase, runtime, read| {
                runtime.change_document_history_bookmark::<H, N>(
                    phase,
                    &program_publication_access(),
                    &self.owner,
                    &self.access,
                    read,
                    node,
                    pin,
                )
            },
        )
    }
}
