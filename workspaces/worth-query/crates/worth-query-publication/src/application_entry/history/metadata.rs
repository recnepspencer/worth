//! Metadata admission uses the current document READ and an independent disclosure decision.
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;

pub(super) fn with_metadata_read<S, H, T>(
    phase: &WorthQueryAdvancementPhase<'_>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    access: H::Access,
    payload: &impl WorthQueryHistoryPayloadMode<S, H>,
    perform: impl for<'read> FnOnce(
        &WorthQueryPrimaryGraphApplicationRuntime<S>,
        &WorthQueryInstalledDocumentHistory<S, H>,
        WorthQueryDocumentHistoryRead<'read, S, H>,
    ) -> Result<T, WorthQueryDocumentHistoryDenial>,
) -> Result<T, WorthQueryApplicationHistoryDenial>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
{
    let installed = request
        .application
        .installed_schema()
        .installed_history::<H>()
        .map_err(WorthQueryApplicationHistoryDenial::Installation)?;
    payload
        .with_read(
            request.query(access),
            phase,
            None,
            |runtime, binding, external, plan| {
                Ok(perform(
                    runtime,
                    &installed,
                    WorthQueryDocumentHistoryRead::new(&installed, binding, external, plan),
                )
                .map_err(WorthQueryApplicationHistoryDenial::Document))
            },
        )
        .map_err(WorthQueryApplicationHistoryDenial::Query)?
}

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
    /// Read selected step metadata when the installed metadata contract is Public.
    pub fn current_step(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryStep, WorthQueryApplicationHistoryDenial> {
        self.current_step_in(request, Ungoverned)
    }

    /// Fresh metadata capability authority is separate from payload READ and edit authority.
    pub fn current_step_governed<C, O, I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> Result<WorthQueryHistoryStep, WorthQueryApplicationHistoryDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.current_step_in(request, Governed { capability, operation, input })
    }

    fn current_step_in(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        authority: impl MetadataAuthority<S, H>,
    ) -> Result<WorthQueryHistoryStep, WorthQueryApplicationHistoryDenial> {
        DocumentRequestContext::bind(&self.owner, request)?.with_metadata(
            self.access.clone(),
            &self.payload,
            authority,
            |phase, runtime, read| {
                runtime.read_document_history_step(
                    phase,
                    &program_publication_access(),
                    &self.owner,
                    read,
                )
            },
        )
    }
}
