use std::num::NonZeroUsize;

use crate::domain_computation::{publish_application_result, WorthQueryPublishedApplicationResult};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode,
    WorthQueryProductQueryControls,
};

use super::WorthQueryApplicationRequestQueryDenial;

pub struct WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent> {
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &'scope WorthQueryRequestScope,
    intent: Intent,
    limits: Option<(NonZeroUsize, NonZeroUsize)>,
}

impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
{
    pub(super) const fn new(
        application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope WorthQueryRequestScope,
        intent: Intent,
    ) -> Self {
        Self {
            application,
            principal,
            scope,
            intent,
            limits: None,
        }
    }

    /// Requests narrower ceilings. Values above the installed binding ceiling
    /// are denied before admission.
    pub fn limits(mut self, maximum_results: NonZeroUsize, maximum_work: NonZeroUsize) -> Self {
        self.limits = Some((maximum_results, maximum_work));
        self
    }

    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<
            <Intent::Binding as ApplicationQueryBinding<Schema>>::Query,
            <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
        >,
        WorthQueryApplicationRequestQueryDenial,
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
        let parameters = self.intent.parameters();
        let scope_binding = self.intent.into_scope();
        let binding = self
            .application
            .installed_schema()
            .installed_query_binding::<Intent::Binding>()
            .map_err(WorthQueryApplicationRequestQueryDenial::BindingInstallation)?;
        let limits = match self.limits {
            Some((maximum_results, maximum_work)) => binding
                .limits()
                .narrow(maximum_results, maximum_work)
                .map_err(WorthQueryApplicationRequestQueryDenial::Limit)?,
            None => binding.limits(),
        };
        let selected = self
            .application
            .on_branch(self.application.current_world())
            .select()
            .map_err(WorthQueryApplicationRequestQueryDenial::ProductSelection)?;
        let principal = selected
            .resolve_authenticated_principal(
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
        let maximum_work = limits.maximum_work();
        let controls =
            WorthQueryProductQueryControls::new(maximum_results, maximum_work, self.scope);
        let plan = selected
            .admit_application_query(binding.query(), &access, parameters, controls)
            .map_err(WorthQueryApplicationRequestQueryDenial::Admission)?;
        let result = self
            .application
            .execute_application_query_one_shot(plan)
            .map_err(WorthQueryApplicationRequestQueryDenial::Execution)?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
    }
}
