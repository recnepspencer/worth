use std::num::NonZeroUsize;

use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_query::{
        ApplicationLiveQueryIntent, ApplicationQueryBinding, ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationLiveControls, WorthQueryApplicationProjection,
    WorthQueryApprovedElevation, WorthQueryPrincipalResolutionMode,
};

use super::WorthQueryApplicationQueryRequest;
use crate::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationLiveOpenRequestDenial,
    WorthQueryApplicationLiveSubscription,
};

type Binding<Schema, Intent> =
    <Intent as worth_query_declaration::facade::application_query::ApplicationQueryIntent<
        Schema,
    >>::Binding;
type Query<Schema, Intent> = <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Query;
type QueryResult<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Scope<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding as worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding<Schema>>::Scope;

impl<'application, Schema, Intent>
    WorthQueryApplicationQueryRequest<'application, '_, '_, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationLiveQueryIntent<Schema>,
    <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    QueryResult<Schema, Intent>: WorthQueryApplicationProjection<Schema, Query<Schema, Intent>>,
{
    pub fn subscribe_approved<Capability, Operation, Input>(
        self,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        live_limits: WorthQueryApplicationLiveLimits,
    ) -> Result<
        WorthQueryApplicationLiveSubscription<'application, Schema, Intent>,
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
            live_limits.maximum_work,
        )
        .map_err(WorthQueryApplicationLiveOpenRequestDenial::Controls)?;
        let maximum_results = NonZeroUsize::new(live_limits.maximum_results)
            .expect("validated live controls reject zero result limits");
        let maximum_work = NonZeroUsize::new(live_limits.maximum_work)
            .expect("validated live controls reject zero work limits");
        let binding_limits = match self.limits {
            Some((results, work)) => binding
                .limits()
                .narrow(results, work)
                .map_err(WorthQueryApplicationLiveOpenRequestDenial::Limit)?,
            None => binding.limits(),
        };
        binding_limits
            .narrow(maximum_results, maximum_work)
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::Limit)?;
        let selected = self
            .application
            .on_branch(self.branch)
            .select()
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::ProductSelection)?;
        let installed_capability = self
            .application
            .installed_schema()
            .capability(capability, operation)
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::CapabilityInstallation)?;
        let principal = selected
            .resolve_authenticated_principal(
                binding.principal_binding(),
                self.principal,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::PrincipalResolution)?;
        let capability_access = selected
            .admit_approved_elevation_access(
                approved,
                &principal,
                &installed_capability,
                input,
                self.scope,
            )
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::CapabilityAdmission)?;
        let (scope_field, scope_value) =
            scope_binding.into_field_parts(principal.principal_identity());
        let scope = selected
            .resolve_entity(
                scope_field,
                scope_value,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::ScopeResolution)?;
        let lease = selected
            .open_governed_application_query_live::<
                <Intent::Binding as ApplicationQueryBinding<Schema>>::Query,
                <<Intent::Binding as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
                <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::Principal,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
                Scope<Schema, Intent>,
                Intent::Target,
                Intent::LiveCause,
                Capability,
                Operation,
                Input,
            >(
                binding.into_query(),
                &principal,
                scope,
                capability_access,
                parameters,
                controls,
            )
            .map_err(WorthQueryApplicationLiveOpenRequestDenial::Open)?;
        Ok(WorthQueryApplicationLiveSubscription::new(
            self.application,
            self.branch,
            lease,
        ))
    }
}
