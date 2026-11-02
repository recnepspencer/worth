//! Fresh metadata decorates the original READ; output custody never enters this core.
use super::*;
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(super) fn with_retention_read<S, H, T>(
    owner: &WorthQueryDocumentHistory<'_, S, H>,
    history_access: &H::Access,
    payload: &impl WorthQueryHistoryPayloadMode<S, H>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    prepare: impl for<'read> FnOnce(
        &WorthQueryAdvancementPhase<'_>,
        &WorthQueryPrimaryGraphApplicationRuntime<S>,
        WorthQueryDocumentHistoryRead<'read, S, H>,
    ) -> Result<
        WorthQueryDocumentHistoryRead<'read, S, H>,
        WorthQueryDocumentHistoryDenial,
    >,
    perform: impl for<'read> FnOnce(
        &WorthQueryAdvancementPhase<'_>,
        &WorthQueryPrimaryGraphApplicationRuntime<S>,
        &WorthQueryDocumentHistory<'_, S, H>,
        &H::Access,
        WorthQueryDocumentHistoryRead<'read, S, H>,
    ) -> Result<T, WorthQueryDocumentHistoryDenial>,
) -> Result<T, WorthQueryApplicationHistoryDenial>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
{
    let runtime = owner.application();
    if !std::ptr::eq(runtime, request.application) {
        return Err(WorthQueryApplicationHistoryDenial::ForeignRuntime);
    }
    let current = WorthQueryApplicationRequest {
        application: runtime,
        principal: request.principal,
        scope: request.scope,
        branch: owner.branch(),
    };
    runtime
        .with_application_advancement(request.scope, |phase| {
            super::super::metadata::with_metadata_read::<S, H, _>(
                &phase,
                &current,
                history_access.clone(),
                payload,
                |runtime, _, read| {
                    let read = prepare(&phase, runtime, read)?;
                    perform(&phase, runtime, owner, history_access, read)
                },
            )
        })
        .map_err(WorthQueryApplicationHistoryDenial::ExecutionRequest)?
}

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
        with_retention_read(
            &self.owner,
            &self.access,
            &self.payload,
            request,
            |_, _, read| Ok(read),
            |phase, runtime, owner, _, read| {
                runtime.read_document_history_retention(
                    phase,
                    &program_publication_access(),
                    owner,
                    read,
                )
            },
        )
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
        with_retention_read(
            &self.owner,
            &self.access,
            &self.payload,
            request,
            |phase, runtime, read| {
                read.with_metadata_capability(runtime, phase, capability, operation, input)
            },
            |phase, runtime, owner, _, read| {
                runtime.read_document_history_retention(
                    phase,
                    &program_publication_access(),
                    owner,
                    read,
                )
            },
        )
    }
}
