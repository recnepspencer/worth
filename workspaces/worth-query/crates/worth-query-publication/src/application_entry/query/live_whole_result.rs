use std::num::NonZeroUsize;

use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationLiveControls, WorthQueryApplicationProjection,
    WorthQueryApprovedElevation, WorthQueryPrincipalResolutionMode,
};

use super::WorthQueryApplicationQueryRequest;
use crate::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationLiveOpenRequestDenial,
    WorthQueryApplicationWholeResultLiveSubscription,
};

type Binding<Schema, Intent> = <Intent as ApplicationQueryIntent<Schema>>::Binding;
type Query<Schema, Intent> = <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Query;
type Parameters<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type QueryResult<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Principal<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Principal;
type PrincipalIdentity<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity;
type Scope<Schema, Intent> =
    <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope;

impl<'application, Schema, Intent>
    WorthQueryApplicationQueryRequest<'application, '_, '_, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    QueryResult<Schema, Intent>: WorthQueryApplicationProjection<Schema, Query<Schema, Intent>>,
{
    pub fn subscribe_whole_result(
        self,
        live_limits: WorthQueryApplicationLiveLimits,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveSubscription<'application, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    > {
        let application = self.application;
        let scope = self.scope;
        application
            .with_application_advancement(scope, |phase| {
                self.subscribe_whole_result_in_advancement(&phase, live_limits)
            })
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest)?
    }

    pub fn subscribe_whole_result_in_advancement(
        self,
        phase: &AdvancementPhase<'_>,
        live_limits: WorthQueryApplicationLiveLimits,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveSubscription<'application, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    > {
        if self.retained.is_some() {
            return Err(WorthQueryApplicationLiveOpenRequestDenial::RetainedBasis);
        }
        let parameters = self.intent.parameters();
        let scope_binding = self.intent.into_scope();
        let binding = self
            .application
            .installed_schema()
            .installed_query_binding::<Intent::Binding>()
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::BindingInstallation)?;
        let controls = WorthQueryApplicationLiveControls::bounded(
            self.scope.clone(),
            live_limits.buffer_capacity,
            live_limits.maximum_results,
        )
        .map_err(WorthQueryApplicationLiveOpenRequestDenial::Controls)?;
        let maximum_results = NonZeroUsize::new(live_limits.maximum_results)
            .expect("validated live controls reject zero result limits");
        let binding_limits = match self.limits {
            Some(results) => self
                .application
                .resolve_application_query_limits(binding.limits())
                .narrow(results)
                .map_err(WorthQueryApplicationLiveOpenRequestDenial::Limit)?,
            None => self
                .application
                .resolve_application_query_limits(binding.limits()),
        };
        binding_limits
            .narrow(maximum_results)
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::Limit)?;
        let selected = self
            .application
            .on_branch(self.branch)
            .select(phase)
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::ProductSelection)?;
        let principal = selected
            .resolve_authenticated_principal(
                phase,
                binding.principal_binding(),
                self.principal,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::PrincipalResolution)?;
        let (scope_field, scope_value) =
            scope_binding.into_field_parts(principal.principal_identity());
        let scope = selected
            .resolve_entity(
                phase,
                scope_field,
                scope_value,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::ScopeResolution)?;
        let lease = selected
            .open_application_query_live_whole_result::<
                Query<Schema, Intent>,
                Parameters<Schema, Intent>,
                QueryResult<Schema, Intent>,
                Principal<Schema, Intent>,
                PrincipalIdentity<Schema, Intent>,
                Scope<Schema, Intent>,
            >(phase, binding.into_query(), &principal, scope, parameters, controls)
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::Open)?;
        Ok(WorthQueryApplicationWholeResultLiveSubscription::new(
            self.application,
            self.branch,
            lease,
        ))
    }

    pub fn subscribe_whole_result_governed<Capability, Operation, Input>(
        self,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        live_limits: WorthQueryApplicationLiveLimits,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveSubscription<'application, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        let application = self.application;
        let request_scope = self.scope;
        application
            .with_application_advancement(request_scope, |phase| {
                self.open_whole_result_governed_in_advancement(
                    &phase,
                    None,
                    capability,
                    operation,
                    input,
                    live_limits,
                )
            })
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest)?
    }

    pub fn subscribe_whole_result_governed_in_advancement<Capability, Operation, Input>(
        self,
        phase: &AdvancementPhase<'_>,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        live_limits: WorthQueryApplicationLiveLimits,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveSubscription<'application, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.open_whole_result_governed_in_advancement(
            phase,
            None,
            capability,
            operation,
            input,
            live_limits,
        )
    }

    pub fn subscribe_whole_result_approved<Capability, Operation, Input>(
        self,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        live_limits: WorthQueryApplicationLiveLimits,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveSubscription<'application, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        let application = self.application;
        let request_scope = self.scope;
        application
            .with_application_advancement(request_scope, |phase| {
                self.open_whole_result_governed_in_advancement(
                    &phase,
                    Some(approved),
                    capability,
                    operation,
                    input,
                    live_limits,
                )
            })
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest)?
    }

    #[allow(clippy::too_many_arguments)]
    pub fn subscribe_whole_result_approved_in_advancement<Capability, Operation, Input>(
        self,
        phase: &AdvancementPhase<'_>,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        live_limits: WorthQueryApplicationLiveLimits,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveSubscription<'application, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.open_whole_result_governed_in_advancement(
            phase,
            Some(approved),
            capability,
            operation,
            input,
            live_limits,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn open_whole_result_governed_in_advancement<Capability, Operation, Input>(
        self,
        phase: &AdvancementPhase<'_>,
        approved: Option<&WorthQueryApprovedElevation>,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        live_limits: WorthQueryApplicationLiveLimits,
    ) -> Result<
        WorthQueryApplicationWholeResultLiveSubscription<'application, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        if self.retained.is_some() {
            return Err(WorthQueryApplicationLiveOpenRequestDenial::RetainedBasis);
        }
        let parameters = self.intent.parameters();
        let scope_binding = self.intent.into_scope();
        let binding = self
            .application
            .installed_schema()
            .installed_query_binding::<Intent::Binding>()
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::BindingInstallation)?;
        let controls = WorthQueryApplicationLiveControls::bounded(
            self.scope.clone(),
            live_limits.buffer_capacity,
            live_limits.maximum_results,
        )
        .map_err(WorthQueryApplicationLiveOpenRequestDenial::Controls)?;
        let maximum_results = NonZeroUsize::new(live_limits.maximum_results)
            .expect("validated live controls reject zero result limits");
        let binding_limits = match self.limits {
            Some(results) => self
                .application
                .resolve_application_query_limits(binding.limits())
                .narrow(results)
                .map_err(WorthQueryApplicationLiveOpenRequestDenial::Limit)?,
            None => self
                .application
                .resolve_application_query_limits(binding.limits()),
        };
        binding_limits
            .narrow(maximum_results)
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::Limit)?;
        let selected = self
            .application
            .on_branch(self.branch)
            .select(phase)
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::ProductSelection)?;
        let installed_capability = self
            .application
            .installed_schema()
            .capability(capability, operation)
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::CapabilityInstallation)?;
        let principal = selected
            .resolve_authenticated_principal(
                phase,
                binding.principal_binding(),
                self.principal,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::PrincipalResolution)?;
        let capability_access = match approved {
            Some(approved) => selected.admit_approved_elevation_access(
                phase,
                approved,
                &principal,
                &installed_capability,
                input,
                self.scope,
            ),
            None => selected.admit_capability_access(
                phase,
                &principal,
                &installed_capability,
                input,
                self.scope,
            ),
        }
        .map_err(WorthQueryApplicationLiveOpenRequestDenial::CapabilityAdmission)?;
        let (scope_field, scope_value) =
            scope_binding.into_field_parts(principal.principal_identity());
        let scope = selected
            .resolve_entity(
                phase,
                scope_field,
                scope_value,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::ScopeResolution)?;
        let lease = selected
            .open_governed_application_query_live_whole_result::<
                Query<Schema, Intent>,
                Parameters<Schema, Intent>,
                QueryResult<Schema, Intent>,
                Principal<Schema, Intent>,
                PrincipalIdentity<Schema, Intent>,
                Scope<Schema, Intent>,
                Capability,
                Operation,
                Input,
            >(
                phase,
                binding.into_query(),
                &principal,
                scope,
                capability_access,
                parameters,
                controls,
            )
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::Open)?;
        Ok(WorthQueryApplicationWholeResultLiveSubscription::new(
            self.application,
            self.branch,
            lease,
        ))
    }
}
