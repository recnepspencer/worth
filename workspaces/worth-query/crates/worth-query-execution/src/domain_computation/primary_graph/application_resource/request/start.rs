//! Start maps input once, admits access, then allocates custody before effects.
use super::super::{
    admission,
    occurrence::{OriginalResourceAccess, ResourceLifecycle, ResourceOccurrence},
    selector::WorthQueryApplicationResource,
    stop::WorthQueryResourceStartStop,
};
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{capture_parameters, ApplicationQueryNativeAccess},
    WorthQueryPrincipalResolutionMode,
};
use worth_query_declaration::facade::{
    application_query::ApplicationQueryScopeResolution,
    application_resource::ApplicationResourceInput,
};
use worth_query_installation::facade::WorthQueryResourceAccessPurpose;

impl<Schema: ApplicationSchema> WorthQueryApplicationResourcesRequest<'_, '_, '_, Schema> {
    pub fn start<Binding>(
        &self,
        input: ApplicationResourceInput<Schema, Binding>,
    ) -> Result<
        WorthQueryApplicationResource<Schema, Binding>,
        WorthQueryResourceStartStop<Schema, Binding>,
    >
    where
        Binding: ApplicationResourceBinding<Schema>,
        <Binding::Access as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Binding::Access as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
    {
        self.start_with::<Binding>(&input, &mut None, None, |_, _, _| Ok(None))
    }

    pub fn start_governed<Binding, Capability, Operation, Input>(
        &self,
        input: ApplicationResourceInput<Schema, Binding>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<
        WorthQueryApplicationResource<Schema, Binding>,
        WorthQueryResourceStartStop<Schema, Binding>,
    >
    where
        Binding: ApplicationResourceBinding<Schema>,
        <Binding::Access as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Binding::Access as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.start_with::<Binding>(&input, &mut None, None, |installed, principal, scope| {
            self.pending(installed, principal, scope, capability)
        })
    }

    pub(in crate::domain_computation::primary_graph::application_resource::request) fn start_with<
        Binding,
    >(
        &self,
        input: &ApplicationResourceInput<Schema, Binding>,
        recovery: &mut Option<
            crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle,
        >,
        recovery_authority: Option<
            &crate::domain_computation::application_aftermath::WorthQueryRecoveryInspectAuthority,
        >,
        pending: impl FnOnce(
            &InstalledResourceBinding<Schema, Binding>,
            EntityId,
            EntityId,
        ) -> Result<
            Option<WorthQueryPendingApplicationQueryGovernance>,
            ResourceCause,
        >,
    ) -> Result<
        WorthQueryApplicationResource<Schema, Binding>,
        WorthQueryResourceStartStop<Schema, Binding>,
    >
    where
        Binding: ApplicationResourceBinding<Schema>,
        <Binding::Access as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Binding::Access as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
    {
        // This outer cell retains the same issued handle even if advancement's
        // post-callback cancellation/deadline prevents an ordinary return.
        let mut handle = None;
        let result = self.open(|execution| {
            let installed = self.installed::<Binding>()?;
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
            execution.consult().map_err(ResourceCause::Request)?;
            let (field, value) =
                Binding::scope(input).into_field_parts(principal.principal().principal_identity());
            let scope = selected
                .resolve_query_scope_issued_with_native_access(
                    execution,
                    field,
                    value,
                    self.request,
                    &mut ApplicationQueryNativeAccess::acquire(),
                )
                .map_err(ResourceCause::Scope)?;
            let parameters = capture_parameters::<Schema, Binding::Access>(
                execution,
                self.runtime,
                std::mem::size_of::<OriginalResourceAccess>() + 2 * std::mem::size_of::<usize>(),
                || Ok(Binding::parameters(input)),
            )
            .map_err(ResourceCause::ParameterCapture)?;
            let selector = scope
                .scope()
                .retain_scope_selector_funded(execution)
                .map_err(ResourceCause::ScopeCapture)?;
            let encoded = super::super::value_capture::capture_input::<Binding::InputBinding>(
                execution,
                input,
                installed
                    .installed
                    .definition()
                    .parts()
                    .limits
                    .maximum_input_bytes,
            )
            .map_err(ResourceCause::Value)?;
            let pending = pending(
                &installed,
                principal.principal().principal_entity_id(),
                scope.scope().entity_id(),
            )?;
            let purpose = WorthQueryResourceAccessPurpose::Start;
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
            let access = Arc::new(OriginalResourceAccess {
                branch: self.branch,
                scope: selector,
                parameters,
                birth_principal: principal.principal().principal_entity_id(),
            });
            if let Some(handle) = recovery.as_ref() {
                super::super::recovery::require_unresolved(
                    self.runtime,
                    execution,
                    handle,
                    recovery_authority.ok_or(ResourceCause::RecoveryHandoffRequired)?,
                    &access,
                    selected.product(),
                )?;
            }
            let occurrence = ResourceOccurrence {
                access: Arc::clone(&access),
                business_product: selected.product().retained_clone(),
                input: encoded,
                lifecycle: ResourceLifecycle::Reserved { clock: None },
                last_clock: None,
                native_slot: None,
                attempt: None,
                provider_closed: true,
                timeout: None,
                retry: None,
                candidate: None,
                result: None,
                recovery: None,
                last_action: None,
            };
            let mut row = installed
                .storage
                .reserve(occurrence)
                .map_err(|(_, cause)| ResourceCause::Storage(cause))?;
            row.occurrence_mut().recovery = recovery.take();
            handle = Some(WorthQueryApplicationResource::issued(
                &installed,
                row.key(),
                access,
            ));
            let result = super::super::progression::start(
                self.runtime,
                execution,
                &installed,
                &mut row,
                input,
                recovery_authority,
            );
            let terminal = admitted.terminal(execution);
            drop(scope);
            drop(principal);
            let (_, product, basis) = selected.into_parts();
            drop(product);
            let terminal = match terminal {
                Ok(value) => value,
                Err(cause) => return Err(super::super::action::prefer_inner(result.err(), cause)),
            };
            let completion = match terminal.complete(execution, basis.release()) {
                Ok(value) => value,
                Err(cause) => return Err(super::super::action::prefer_inner(result.err(), cause)),
            };
            row.occurrence_mut().last_action = Some(completion);
            result
        });
        match result {
            Ok(()) => Ok(handle.expect("successful Start allocated its actual row")),
            Err(cause) => Err(WorthQueryResourceStartStop {
                handle,
                denial: cause.into(),
            }),
        }
    }
}
