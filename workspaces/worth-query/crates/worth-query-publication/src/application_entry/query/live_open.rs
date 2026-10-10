use super::*;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;

impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationLiveQueryIntent<Schema>,
{
    pub fn subscribe(
        self,
        live_limits: super::super::WorthQueryApplicationLiveLimits,
    ) -> Result<
        super::super::WorthQueryApplicationLiveSubscription<'application, Schema, Intent>,
        super::super::WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
            WorthQueryApplicationProjection<
                Schema,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::Query,
            >,
    {
        let application = self.application;
        let scope = self.scope;
        application
            .with_application_advancement(scope, |phase| {
                self.subscribe_in_advancement(&phase, live_limits)
            })
            .map_err(super::super::WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest)?
    }

    pub(in crate::application_entry) fn subscribe_in_advancement(
        self,
        phase: &AdvancementPhase<'_>,
        live_limits: super::super::WorthQueryApplicationLiveLimits,
    ) -> Result<
        super::super::WorthQueryApplicationLiveSubscription<'application, Schema, Intent>,
        super::super::WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
            WorthQueryApplicationProjection<
                Schema,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::Query,
            >,
    {
        if self.retained.is_some() {
            return Err(super::super::WorthQueryApplicationLiveOpenRequestDenial::RetainedBasis);
        }
        let parameters = self.intent.parameters();
        let scope_binding = self.intent.into_scope();
        let binding = self
            .application
            .installed_schema()
            .installed_query_binding::<Intent::Binding>()
            .map_err(
                super::super::WorthQueryApplicationLiveOpenRequestDenial::BindingInstallation,
            )?;
        let controls = worth_query_execution::facade::primary_graph::WorthQueryApplicationLiveControls::bounded(
            self.scope.clone(),
            live_limits.buffer_capacity,
            live_limits.maximum_results,
            live_limits.maximum_work,
        )
        .map_err(super::super::WorthQueryApplicationLiveOpenRequestDenial::Controls)?;
        let maximum_results = NonZeroUsize::new(live_limits.maximum_results)
            .expect("validated live controls reject zero result limits");
        let maximum_work = NonZeroUsize::new(live_limits.maximum_work)
            .expect("validated live controls reject zero work limits");
        let binding_limits = match self.limits {
            Some((results, work)) => self
                .application
                .resolve_application_query_limits(binding.limits())
                .narrow(results, work)
                .map_err(super::super::WorthQueryApplicationLiveOpenRequestDenial::Limit)?,
            None => self
                .application
                .resolve_application_query_limits(binding.limits()),
        };
        binding_limits
            .narrow(maximum_results, maximum_work)
            .map_err(super::super::WorthQueryApplicationLiveOpenRequestDenial::Limit)?;
        let selected =
            self.application.on_branch(self.branch).select().map_err(
                super::super::WorthQueryApplicationLiveOpenRequestDenial::ProductSelection,
            )?;
        let principal = selected
            .resolve_authenticated_principal(
                binding.principal_binding(),
                self.principal,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(
                super::super::WorthQueryApplicationLiveOpenRequestDenial::PrincipalResolution,
            )?;
        let (scope_field, scope_value) =
            scope_binding.into_field_parts(principal.principal_identity());
        let scope = selected
            .resolve_entity(
                scope_field,
                scope_value,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(super::super::WorthQueryApplicationLiveOpenRequestDenial::ScopeResolution)?;
        let lease = selected
            .open_application_query_live::<
                <Intent::Binding as ApplicationQueryBinding<Schema>>::Query,
                <<Intent::Binding as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
                <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::Principal,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
                <<Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding<Schema>>::Scope,
                Intent::Target,
                Intent::LiveCause,
            >(phase, binding.into_query(), &principal, scope, parameters, controls)
            .map_err(super::super::WorthQueryApplicationLiveOpenRequestDenial::Open)?;
        Ok(super::super::WorthQueryApplicationLiveSubscription::new(
            self.application,
            self.branch,
            lease,
        ))
    }
}
