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
        self.document.maintain_history(request)?;
        let target = self.target;
        read::with_retention_read(
            &self.document.owner,
            &self.document.access,
            &self.document.payload,
            request,
            |_, _, read| Ok(read),
            |phase, runtime, owner, access, read| {
                runtime.change_document_history_window::<H, N>(
                    phase,
                    &program_publication_access(),
                    owner,
                    access,
                    read,
                    target,
                )
            },
        )
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
        self.document.maintain_history(request)?;
        let target = self.target;
        read::with_retention_read(
            &self.document.owner,
            &self.document.access,
            &self.document.payload,
            request,
            |phase, runtime, read| {
                read.with_metadata_capability(runtime, phase, capability, operation, input)
            },
            |phase, runtime, owner, access, read| {
                runtime.change_document_history_window::<H, N>(
                    phase,
                    &program_publication_access(),
                    owner,
                    access,
                    read,
                    target,
                )
            },
        )
    }
}
