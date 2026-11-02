//! A stored purpose is a recipe for fresh admission, never retained authority.
use super::*;
use crate::application_entry::query::admitted::RequestPlan;
use crate::application_entry::{
    WorthQueryApplicationQueryRequest, WorthQueryApplicationRequestQueryDenial,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
use worth_query_execution::{
    facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
    publication_boundary::document_history::WorthQueryHistoryPreviewReadAdmission,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

/// Sealed payload admission mode carried by the owned document and its edits.
/// A purpose does not retain a principal, approval, grant, or admitted query.
/// A generic transfer preserves the original mode:
/// ```
/// use worth_query_publication::facade::application_entry::{WorthQueryApplicationHistoryDocument, WorthQueryHistoryPayloadMode};
/// use worth_query_declaration::facade::{application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema};
/// fn transfer<S: ApplicationSchema, H: ApplicationHistoryBinding<S>, P: WorthQueryHistoryPayloadMode<S, H>>(
///     document: WorthQueryApplicationHistoryDocument<'_, S, H, P>,
/// ) -> WorthQueryApplicationHistoryDocument<'_, S, H, P> { document }
/// ```
/// It cannot erase the purpose by returning an ordinary-mode owner:
/// ```compile_fail,E0308
/// use worth_query_publication::facade::application_entry::{WorthQueryApplicationHistoryDocument, WorthQueryHistoryPayloadMode};
/// use worth_query_declaration::facade::{application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema};
/// fn erase<S: ApplicationSchema, H: ApplicationHistoryBinding<S>, P: WorthQueryHistoryPayloadMode<S, H>>(
///     document: WorthQueryApplicationHistoryDocument<'_, S, H, P>,
/// ) -> WorthQueryApplicationHistoryDocument<'_, S, H> { document }
/// ```
#[allow(private_bounds)]
pub trait WorthQueryHistoryPayloadMode<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>:
    PayloadDriver<S, H>
{
}

/// Use the ordinary installed query policy on every read.
#[derive(Debug)]
pub struct OrdinaryHistoryPayload;

/// Repeat the original governed query admission with this purpose and fresh authority.
/// Constructed by `attach_history_with_payload`; capability references are not grants.
/// ```compile_fail,E0451
/// use worth_query_publication::facade::application_entry::WorthQueryGovernedHistoryPayload;
/// use worth_query_declaration::facade::{application_capability::ApplicationCapabilityRef, application_schema::ApplicationOperationRef};
/// fn forge<S, C, O, I>(capability: ApplicationCapabilityRef<S, C>, operation: ApplicationOperationRef<S, O, I>, input: I)
///     -> WorthQueryGovernedHistoryPayload<S, C, O, I> {
///     WorthQueryGovernedHistoryPayload { capability, operation, input }
/// }
/// ```
pub struct WorthQueryGovernedHistoryPayload<S, C, O, I> {
    pub(super) capability: ApplicationCapabilityRef<S, C>,
    pub(super) operation: ApplicationOperationRef<S, O, I>,
    pub(super) input: I,
}

pub(super) trait PayloadDriver<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> {
    fn with_read<T>(
        &self,
        query: WorthQueryApplicationQueryRequest<'_, '_, '_, S, H::Access>,
        phase: &WorthQueryAdvancementPhase<'_>,
        preview: Option<&WorthQueryHistoryPreviewReadAdmission<'_, '_>>,
        perform: impl FnOnce(
            &WorthQueryPrimaryGraphApplicationRuntime<S>,
            &WorthQueryInstalledApplicationQueryBinding<S, Binding<S, H>>,
            &WorthQueryAuthenticatedExternalPrincipal<S>,
            RequestPlan<'_, S, Binding<S, H>>,
        ) -> Result<T, WorthQueryApplicationRequestQueryDenial>,
    ) -> Result<T, WorthQueryApplicationRequestQueryDenial>
    where
        Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>;
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryHistoryPayloadMode<S, H>
    for OrdinaryHistoryPayload
{
}
impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> PayloadDriver<S, H>
    for OrdinaryHistoryPayload
{
    fn with_read<T>(
        &self,
        query: WorthQueryApplicationQueryRequest<'_, '_, '_, S, H::Access>,
        phase: &WorthQueryAdvancementPhase<'_>,
        preview: Option<&WorthQueryHistoryPreviewReadAdmission<'_, '_>>,
        perform: impl FnOnce(
            &WorthQueryPrimaryGraphApplicationRuntime<S>,
            &WorthQueryInstalledApplicationQueryBinding<S, Binding<S, H>>,
            &WorthQueryAuthenticatedExternalPrincipal<S>,
            RequestPlan<'_, S, Binding<S, H>>,
        ) -> Result<T, WorthQueryApplicationRequestQueryDenial>,
    ) -> Result<T, WorthQueryApplicationRequestQueryDenial>
    where
        Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    {
        query.with_admitted_preview_query(phase, preview, perform)
    }
}

impl<S, H, C, O, I> WorthQueryHistoryPayloadMode<S, H>
    for WorthQueryGovernedHistoryPayload<S, C, O, I>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    O: ApplicationOperationMarkerIdentity<S> + 'static,
    O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
    I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + Clone + 'static,
{
}

impl<S, H, C, O, I> PayloadDriver<S, H> for WorthQueryGovernedHistoryPayload<S, C, O, I>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    O: ApplicationOperationMarkerIdentity<S> + 'static,
    O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
    I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + Clone + 'static,
{
    fn with_read<T>(
        &self,
        query: WorthQueryApplicationQueryRequest<'_, '_, '_, S, H::Access>,
        phase: &WorthQueryAdvancementPhase<'_>,
        preview: Option<&WorthQueryHistoryPreviewReadAdmission<'_, '_>>,
        perform: impl FnOnce(
            &WorthQueryPrimaryGraphApplicationRuntime<S>,
            &WorthQueryInstalledApplicationQueryBinding<S, Binding<S, H>>,
            &WorthQueryAuthenticatedExternalPrincipal<S>,
            RequestPlan<'_, S, Binding<S, H>>,
        ) -> Result<T, WorthQueryApplicationRequestQueryDenial>,
    ) -> Result<T, WorthQueryApplicationRequestQueryDenial>
    where
        Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    {
        query.with_admitted_governed_preview_query(
            phase,
            preview,
            None,
            self.capability,
            self.operation,
            self.input.clone(),
            perform,
        )
    }
}
