//! Every entry repeats the original current payload and metadata policies.
use super::*;

pub(super) fn with_read<S, H, P, Outputs, T>(
    document: &WorthQueryApplicationHistoryDocument<'_, S, H, P, Outputs>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    authority: impl MetadataAuthority<S, H>,
    perform: impl for<'read> FnOnce(
        &WorthQueryAdvancementPhase<'_>,
        &WorthQueryPrimaryGraphApplicationRuntime<S>,
        WorthQueryDocumentHistoryRead<'read, S, H>,
    ) -> Result<T, WorthQueryHistoryNavigationAdvanceDenial>,
) -> Result<T, WorthQueryHistoryNavigationAdvanceDenial>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
{
    DocumentRequestContext::bind(&document.owner, request)
        .and_then(|context| {
            context.with_metadata(
                document.access.clone(),
                &document.payload,
                authority,
                |phase, runtime, read| Ok(perform(phase, runtime, read)),
            )
        })
        .map_err(WorthQueryHistoryNavigationAdvanceDenial::History)?
}

impl<S, H, P, Outputs> WorthQueryApplicationHistoryDocument<'_, S, H, P, Outputs>
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
        self.selection_in(request, Ungoverned)
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
        self.selection_in(request, Governed { capability, operation, input })
    }

    fn selection_in(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        authority: impl MetadataAuthority<S, H>,
    ) -> Result<WorthQueryHistorySelection, WorthQueryHistoryNavigationAdvanceDenial> {
        with_read(self, request, authority, |phase, runtime, read| {
            runtime
                .read_document_history_selection(
                    phase,
                    &program_publication_access(),
                    &self.owner,
                    read,
                )
                .map_err(WorthQueryHistoryNavigationDenial::Document)
                .map_err(WorthQueryHistoryNavigationAdvanceDenial::Navigation)
        })
    }
}
