use std::num::NonZeroUsize;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;

use crate::domain_computation::{publish_application_result, WorthQueryPublishedApplicationResult};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationLiveQueryIntent, ApplicationQueryBinding, ApplicationQueryIntent,
        ApplicationQueryScopeResolution,
    },
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode,
    WorthQueryProductQueryControls,
};

use super::WorthQueryApplicationRequestQueryDenial;

mod admitted;
mod continuation;
mod governed;
mod governed_continuation;
mod governed_retained;
mod live_approved;
mod live_open;
mod managed_index;

/// A query request. `execute` runs it once; `subscribe` opens it live; `limits` bounds its
/// results and work.
pub struct WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent> {
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &'scope WorthQueryRequestScope,
    branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    retained: Option<
        std::sync::Arc<
            worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
        >,
    >,
    intent: Intent,
    limits: Option<NonZeroUsize>,
    retain_expression_context: bool,
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
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        intent: Intent,
    ) -> Self {
        Self {
            application,
            principal,
            scope,
            branch,
            retained: None,
            intent,
            limits: None,
            retain_expression_context: false,
        }
    }

    pub(super) fn new_at(
        application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope WorthQueryRequestScope,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        observation: std::sync::Arc<
            worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
        >,
        intent: Intent,
    ) -> Self {
        Self {
            application,
            principal,
            scope,
            branch,
            retained: Some(observation),
            intent,
            limits: None,
            retain_expression_context: false,
        }
    }

    /// Requests narrower ceilings. Values above the installed binding ceiling
    /// are denied before admission.
    pub fn limit_results(mut self, maximum_results: NonZeroUsize) -> Self {
        self.limits = Some(maximum_results);
        self
    }

    /// Executes an exactly-one query and retains its native expression record
    /// on the observed source, before the application result projector runs.
    /// Incomplete disclosure or unsupported scalar/unit meaning refuses the
    /// request. The context grants no read or mutation permission.
    pub fn execute_with_expression_context(
        mut self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<
            <Intent::Binding as ApplicationQueryBinding<Schema>>::Query,
            <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
        >,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
        <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
            WorthQueryApplicationProjection<Schema, <Intent::Binding as ApplicationQueryBinding<Schema>>::Query>,
{
        self.retain_expression_context = true;
        self.execute()
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
        let application = self.application;
        let scope = self.scope;
        application
            .with_application_advancement(scope, |phase| self.execute_in_advancement(&phase))
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }

    pub fn execute_in_advancement(
        self,
        phase: &AdvancementPhase<'_>,
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
        self.with_admitted_query(phase, |application, binding, external, plan| {
            admitted::execute_admitted_query(phase, application, binding, external, plan)
        })
    }
}
