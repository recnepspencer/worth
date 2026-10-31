//! Every entry repeats the original current payload and metadata policies.
use super::*;

pub(super) fn with_read<S, H, P, T>(
    document: &WorthQueryApplicationHistoryDocument<'_, S, H, P>,
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
        WorthQueryDocumentHistoryRead<'read, S, H>,
    ) -> Result<T, WorthQueryHistoryNavigationDenial>,
) -> Result<T, WorthQueryHistoryNavigationAdvanceDenial>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
{
    use WorthQueryHistoryNavigationAdvanceDenial as Denial;
    let runtime = document.owner.application();
    if !std::ptr::eq(runtime, request.application) {
        return Err(Denial::History(
            WorthQueryApplicationHistoryDenial::ForeignRuntime,
        ));
    }
    let current = WorthQueryApplicationRequest {
        application: runtime,
        principal: request.principal,
        scope: request.scope,
        branch: document.owner.branch(),
    };
    runtime
        .with_application_advancement(request.scope, |phase| {
            super::super::metadata::with_metadata_read::<S, H, _>(
                &phase,
                &current,
                document.access.clone(),
                &document.payload,
                |runtime, _, read| {
                    let read = prepare(&phase, runtime, read)?;
                    Ok(perform(&phase, runtime, read))
                },
            )
        })
        .map_err(|cause| {
            Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
        })?
        .map_err(Denial::History)?
        .map_err(Denial::Navigation)
}

impl<S, H, P> WorthQueryApplicationHistoryDocument<'_, S, H, P>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    /// Fresh READ and metadata disclose the selected occurrence and actual
    /// writable context. The descriptor grants no navigation permission.
    pub fn selection(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistorySelection, WorthQueryHistoryNavigationAdvanceDenial> {
        self.selection_with(request, |_, _, read| Ok(read))
    }

    pub fn selection_governed<C, O, I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> Result<WorthQueryHistorySelection, WorthQueryHistoryNavigationAdvanceDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.selection_with(request, |phase, runtime, read| {
            read.with_metadata_capability(runtime, phase, capability, operation, input)
        })
    }

    fn selection_with(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        prepare: impl for<'read> FnOnce(
            &WorthQueryAdvancementPhase<'_>,
            &WorthQueryPrimaryGraphApplicationRuntime<S>,
            WorthQueryDocumentHistoryRead<'read, S, H>,
        ) -> Result<
            WorthQueryDocumentHistoryRead<'read, S, H>,
            WorthQueryDocumentHistoryDenial,
        >,
    ) -> Result<WorthQueryHistorySelection, WorthQueryHistoryNavigationAdvanceDenial> {
        with_read(self, request, prepare, |phase, runtime, read| {
            runtime
                .read_document_history_selection(
                    phase,
                    &program_publication_access(),
                    &self.owner,
                    read,
                )
                .map_err(WorthQueryHistoryNavigationDenial::Document)
        })
    }
}
