//! One private admission engine for execute, open, and explicit cold rebuild.
use super::*;
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationQueryPlan;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

pub(super) type RequestPlan<'a, S, B> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, <B as ApplicationQueryBinding<S>>::Query,
    <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    <B as ApplicationQueryBinding<S>>::Principal,
    <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;
impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
{
    pub(super) fn with_admitted_query<T>(
        self,
        phase: &AdvancementPhase<'_>,
        perform: impl FnOnce(
            &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            &WorthQueryInstalledApplicationQueryBinding<Schema, Intent::Binding>,
            &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            RequestPlan<'_, Schema, Intent::Binding>,
        ) -> Result<T, WorthQueryApplicationRequestQueryDenial>,
    ) -> Result<T, WorthQueryApplicationRequestQueryDenial>
    where
        <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
    {
        let parameters = self.intent.parameters();
        let scope_binding = self.intent.into_scope();
        let binding = self
            .application
            .installed_schema()
            .installed_query_binding::<Intent::Binding>()
            .map_err(WorthQueryApplicationRequestQueryDenial::BindingInstallation)?;
        let limits = match self.limits {
            Some(maximum_results) => self
                .application
                .resolve_application_query_limits(binding.limits())
                .narrow(maximum_results)
                .map_err(WorthQueryApplicationRequestQueryDenial::Limit)?,
            None => self
                .application
                .resolve_application_query_limits(binding.limits()),
        };
        let selected = self
            .application
            .on_branch(self.branch)
            .select(phase)
            .map_err(WorthQueryApplicationRequestQueryDenial::ProductSelection)?;
        let retained = match self.retained.as_ref() {
            Some(observation) => Some(
                self.application
                    .select_application_read_observation(phase, observation)
                    .map_err(WorthQueryApplicationRequestQueryDenial::ProductSelection)?,
            ),
            None => None,
        };
        let principal = selected
            .resolve_authenticated_principal(
                phase,
                binding.principal_binding(),
                self.principal,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::PrincipalResolution)?;
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
            .map_err(WorthQueryApplicationRequestQueryDenial::ScopeResolution)?;
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let query_maximum = NonZeroUsize::new(binding.query().read_graph().maximum_result_count())
            .unwrap_or(NonZeroUsize::MAX);
        let maximum_results = limits.maximum_results().min(query_maximum);

        let controls = WorthQueryProductQueryControls::new(maximum_results, self.scope);
        let plan = match retained {
            Some(retained) => selected.admit_retained_application_query(
                phase,
                retained,
                binding.query(),
                &access,
                parameters,
                controls,
            ),
            None => selected.admit_application_query(
                phase,
                binding.query(),
                &access,
                parameters,
                controls,
            ),
        }
        .map_err(WorthQueryApplicationRequestQueryDenial::Admission)?;
        let plan = if self.retain_expression_context {
            plan.retain_expression_context()
        } else {
            plan
        };
        perform(self.application, &binding, self.principal, plan)
    }
}
