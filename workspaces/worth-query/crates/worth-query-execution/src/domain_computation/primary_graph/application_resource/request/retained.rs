//! Cold logical custody uses fresh installed Resource actions and facets.
use super::super::{admission, stop::WorthQueryResourceActionDenial, value_capture};
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{capture_parameters, ApplicationQueryNativeAccess},
    query_obligations::{QueryResourceObligation, QueryResourceResultKind},
    WorthQueryPrincipalResolutionMode,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::OwnedAsyncObligationIdentity;
use worth_query_declaration::facade::{
    application_query::ApplicationQueryScopeResolution,
    application_resource::ApplicationResourceInput,
    application_schema::ApplicationScalarValueBinding,
};
use worth_query_installation::facade::WorthQueryResourceAccessPurpose as Purpose;

pub use crate::domain_computation::primary_graph::query_obligations::QueryOwnedAsyncTerminal as WorthQueryResourceTerminal;
/// Descriptive original identity and actual recorded release posture only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRetainedResourceState {
    pub identity: OwnedAsyncObligationIdentity,
    pub released: bool,
    pub terminal: Option<WorthQueryResourceTerminal>,
}
type ResultValue<S, B> =
    <<B as ApplicationResourceBinding<S>>::ResultBinding as ApplicationScalarValueBinding>::Value;
impl<S: ApplicationSchema> WorthQueryApplicationResourcesRequest<'_, '_, '_, S> {
    pub fn retained_state<B>(
        &self,
        input: &ApplicationResourceInput<S, B>,
        identity: OwnedAsyncObligationIdentity,
    ) -> Result<WorthQueryRetainedResourceState, WorthQueryResourceActionDenial>
    where
        B: ApplicationResourceBinding<S>,
        <B::Access as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
            S,
            <B::Access as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
    {
        self.with_retained::<B, _>(
            input,
            identity,
            Purpose::State,
            |_, _, _| Ok(None),
            |_, row| Ok(state(row)),
        )
    }
    pub fn with_retained_result<B, R>(
        &self,
        input: &ApplicationResourceInput<S, B>,
        identity: OwnedAsyncObligationIdentity,
        inspect: impl FnOnce(&ResultValue<S, B>) -> R,
    ) -> Result<Option<R>, WorthQueryResourceActionDenial>
    where
        B: ApplicationResourceBinding<S>,
        <B::Access as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
            S,
            <B::Access as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
    {
        self.with_retained::<B, _>(
            input,
            identity,
            Purpose::Result,
            |_, _, _| Ok(None),
            |execution, row| read_result::<S, B, R>(execution, row, inspect),
        )
    }
    pub fn retained_state_governed<B, Capability, Operation, Input>(
        &self,
        input: &ApplicationResourceInput<S, B>,
        identity: OwnedAsyncObligationIdentity,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<S, Capability, Operation, Input>,
    ) -> Result<WorthQueryRetainedResourceState, WorthQueryResourceActionDenial>
    where
        B: ApplicationResourceBinding<S>,
        <B::Access as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
            S,
            <B::Access as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
        Input: ApplicationCapabilityRequest<S, Capability, Scope = AccessScope<S, B>>,
    {
        self.with_retained::<B, _>(
            input,
            identity,
            Purpose::State,
            |installed, principal, scope| self.pending(installed, principal, scope, capability),
            |_, row| Ok(state(row)),
        )
    }
    pub fn with_retained_result_governed<B, Capability, Operation, Input, R>(
        &self,
        input: &ApplicationResourceInput<S, B>,
        identity: OwnedAsyncObligationIdentity,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<S, Capability, Operation, Input>,
        inspect: impl FnOnce(&ResultValue<S, B>) -> R,
    ) -> Result<Option<R>, WorthQueryResourceActionDenial>
    where
        B: ApplicationResourceBinding<S>,
        <B::Access as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
            S,
            <B::Access as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
        Input: ApplicationCapabilityRequest<S, Capability, Scope = AccessScope<S, B>>,
    {
        self.with_retained::<B, _>(
            input,
            identity,
            Purpose::Result,
            |installed, principal, scope| self.pending(installed, principal, scope, capability),
            |execution, row| read_result::<S, B, R>(execution, row, inspect),
        )
    }
    pub fn discharge_retained<B>(
        &self,
        input: &ApplicationResourceInput<S, B>,
        identity: OwnedAsyncObligationIdentity,
    ) -> Result<(), WorthQueryResourceActionDenial>
    where
        B: ApplicationResourceBinding<S>,
        <B::Access as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
            S,
            <B::Access as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
    {
        self.with_retained::<B, _>(
            input,
            identity,
            Purpose::Close,
            |_, _, _| Ok(None),
            |execution, _| {
                let installed = self.installed::<B>()?;
                if installed.storage.retains_obligation(identity) {
                    return Err(ResourceCause::Storage(
                        super::super::storage::ResourceStorageDenial::CleanupPending,
                    ));
                }
                self.runtime
                    .installed_program_support()
                    .ok_or(ResourceCause::Unavailable)?
                    .lifecycle()
                    .discharge_resource(execution, identity)
                    .map_err(ResourceCause::Obligation)
            },
        )
    }
    fn with_retained<B, R>(
        &self,
        input: &ApplicationResourceInput<S, B>,
        identity: OwnedAsyncObligationIdentity,
        purpose: Purpose,
        pending: impl FnOnce(
            &InstalledResourceBinding<S, B>,
            EntityId,
            EntityId,
        ) -> Result<
            Option<WorthQueryPendingApplicationQueryGovernance>,
            ResourceCause,
        >,
        action: impl FnOnce(
            &ExecutionRequest<'_, '_>,
            &QueryResourceObligation,
        ) -> Result<R, ResourceCause>,
    ) -> Result<R, WorthQueryResourceActionDenial>
    where
        B: ApplicationResourceBinding<S>,
        <B::Access as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
            S,
            <B::Access as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
    {
        self.open(|execution| {
            let installed = self.installed::<B>()?;
            let selected = self
                .runtime
                .on_branch(self.branch)
                .select_with_request(execution)
                .map_err(ResourceCause::Selection)?;
            let principal = selected
                .resolve_authenticated_principal_issued(
                    execution,
                    installed.installed.access_binding().principal_binding(),
                    self.external,
                    self.request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                    None,
                )
                .map_err(ResourceCause::Principal)?;
            let (field, value) =
                B::scope(input).into_field_parts(principal.principal().principal_identity());
            let scope = selected
                .resolve_query_scope_issued_with_native_access(
                    execution,
                    field,
                    value,
                    self.request,
                    &mut ApplicationQueryNativeAccess::acquire(),
                )
                .map_err(ResourceCause::Scope)?;
            let parameters = capture_parameters::<S, B::Access>(execution, self.runtime, 0, || {
                Ok(B::parameters(input))
            })
            .map_err(ResourceCause::ParameterCapture)?;
            let pending = pending(
                &installed,
                principal.principal().principal_entity_id(),
                scope.scope().entity_id(),
            )?;
            let admitted = admission::prepare(
                self.runtime,
                execution,
                &installed.installed,
                &installed.prepared[&purpose],
                purpose,
                &selected,
                &principal,
                &scope,
                self.request,
                None,
                &parameters,
                pending,
            )
            .map_err(ResourceCause::Admission)?;
            let result = (|| {
                let history = self
                    .runtime
                    .installed_program_support()
                    .ok_or(ResourceCause::Unavailable)?
                    .lifecycle();
                history
                    .settle_resource_publication(execution)
                    .map_err(ResourceCause::Obligation)?;
                let binding = installed.installed.binding_identity();
                let row = history
                    .resource_obligation(identity)
                    .ok_or(ResourceCause::Unavailable)?;
                let original = &row.original;
                if !(original.binding == B::IDENTITY
                    && &original.package == binding.package_identity().bytes()
                    && &original.schema == binding.schema_identity().bytes()
                    && original.branch
                        == selected
                            .product()
                            .observation()
                            .branch_identity()
                            .name()
                            .as_str()
                    && original.scope_subject == scope.scope().entity_id()
                    && &original.query == parameters.query().as_bytes()
                    && original
                        .parameters
                        .iter()
                        .map(|(key, value)| (key.as_str(), value))
                        .eq(parameters
                            .parameters()
                            .bindings()
                            .iter()
                            .map(|(key, value)| (key.as_ref(), value))))
                {
                    return Err(ResourceCause::Unavailable);
                }
                action(execution, &row)
            })();
            let terminal = admitted.terminal(execution);
            drop(scope);
            drop(principal);
            let (_, product, basis) = selected.into_parts();
            drop(product);
            let terminal =
                terminal.map_err(|cause| super::super::action::prefer_inner(None, cause))?;
            terminal
                .complete(execution, basis.release())
                .map_err(|cause| super::super::action::prefer_inner(None, cause))?;
            result
        })
        .map_err(Self::denial)
    }
}

fn state(row: &QueryResourceObligation) -> WorthQueryRetainedResourceState {
    WorthQueryRetainedResourceState {
        identity: row.identity,
        released: row.handoff.release.is_some(),
        terminal: row.handoff.release.as_ref().and_then(|release| {
            release
                .terminal
                .as_ref()
                .and_then(|terminal| match terminal.status() {
                    worth_signal::facade::ResourceInFlightStatus::Active => None,
                    worth_signal::facade::ResourceInFlightStatus::Fulfilled => {
                        Some(WorthQueryResourceTerminal::Fulfilled)
                    }
                    worth_signal::facade::ResourceInFlightStatus::Rejected => {
                        Some(WorthQueryResourceTerminal::Rejected)
                    }
                    worth_signal::facade::ResourceInFlightStatus::Cancelled => {
                        Some(WorthQueryResourceTerminal::Cancelled)
                    }
                    worth_signal::facade::ResourceInFlightStatus::TimedOut => {
                        Some(WorthQueryResourceTerminal::TimedOut)
                    }
                    worth_signal::facade::ResourceInFlightStatus::Superseded => {
                        Some(WorthQueryResourceTerminal::Superseded)
                    }
                })
        }),
    }
}

fn read_result<S: ApplicationSchema, B: ApplicationResourceBinding<S>, R>(
    execution: &ExecutionRequest<'_, '_>,
    row: &QueryResourceObligation,
    inspect: impl FnOnce(&ResultValue<S, B>) -> R,
) -> Result<Option<R>, ResourceCause> {
    let Some((QueryResourceResultKind::Result, value)) = &row.handoff.result else {
        return Ok(None);
    };
    value_capture::with_scalar::<B::ResultBinding, _>(execution, value, inspect)
        .map(Some)
        .map_err(ResourceCause::Value)
}
