//! Each bounded window batch repeats both disclosure and role admission.
use super::*;
impl<S, H, Payload, Outputs> WorthQueryHistoryWindowChange<'_, '_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    /// Exclude at most one old step; dropping this borrow preserves its registered target.
    pub fn advance<N: ApplicationHistoryWindowBinding<S, H>>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial> {
        self.advance_in::<N>(request, Ungoverned)
    }

    pub fn advance_governed<N, C, O, I>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial>
    where
        N: ApplicationHistoryWindowBinding<S, H>,
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.advance_in::<N>(request, Governed { capability, operation, input })
    }

    fn advance_in<N: ApplicationHistoryWindowBinding<S, H>>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        authority: impl MetadataAuthority<S, H>,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial> {
        self.document.maintain_history(request)?;
        let target = self.target;
        DocumentRequestContext::bind(&self.document.owner, request)?.with_metadata(
            self.document.access.clone(),
            &self.document.payload,
            authority,
            |phase, runtime, read| {
                runtime.change_document_history_window::<H, N>(
                    phase,
                    &program_publication_access(),
                    &self.document.owner,
                    &self.document.access,
                    read,
                    target,
                )
            },
        )
    }
}
