//! A complete collection of ordinary queries over one retained data basis.

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
    WorthQueryApplicationProjection, WorthQueryApplicationQueryBatchResourceDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};

mod execution;

/// Ordered ordinary declared reads over one retained data observation.
/// Every item is freshly authorized and keeps its own source/disclosure receipt.
pub struct WorthQueryApplicationQueryBatchRequest<'application, 'principal, 'scope, Schema, Intent>
{
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &'scope WorthQueryRequestScope,
    branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    observation: std::sync::Arc<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
    >,
    intents: Vec<Intent>,
}

/// No denial contains a partial batch or a source from an earlier item.
#[derive(Debug)]
pub enum WorthQueryApplicationQueryBatchDenial {
    Resource {
        item: Option<usize>,
        denial: WorthQueryApplicationQueryBatchResourceDenial,
    },
    Item {
        index: usize,
        cause: super::WorthQueryApplicationRequestQueryDenial,
    },
    Cancelled,
    DeadlineExceeded,
}

impl<
        'application,
        'principal,
        'scope,
        Schema: ApplicationSchema,
        Intent: ApplicationQueryIntent<Schema>,
    > WorthQueryApplicationQueryBatchRequest<'application, 'principal, 'scope, Schema, Intent>
{
    pub(super) fn new(
        application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope WorthQueryRequestScope,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        observation: std::sync::Arc<
            worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
        >,
        intents: Vec<Intent>,
    ) -> Self {
        Self {
            application,
            principal,
            scope,
            branch,
            observation,
            intents,
        }
    }

    pub fn execute(
self, phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationQueryBatch<
            <Intent::Binding as ApplicationQueryBinding<Schema>>::Query,
            <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
        >,
        WorthQueryApplicationQueryBatchDenial,
    >
    where
        <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
        <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <Intent::Binding as ApplicationQueryBinding<Schema>>::Query>,
    {
        self.execute_installed(None, phase)
    }

    pub(super) fn execute_installed(
self, query_limits: Option<std::num::NonZeroUsize>, phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationQueryBatch<
            <Intent::Binding as ApplicationQueryBinding<Schema>>::Query,
            <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
        >,
        WorthQueryApplicationQueryBatchDenial,
    >
    where
        <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
        <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <Intent::Binding as ApplicationQueryBinding<Schema>>::Query>,
    {
        execution::execute(phase, self, query_limits)
    }
}
