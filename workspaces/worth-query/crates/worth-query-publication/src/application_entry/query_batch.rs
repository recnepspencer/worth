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
    WorthQueryApplicationProjection, WorthQueryApplicationQueryBatchAdmission,
    WorthQueryApplicationQueryBatchLimits, WorthQueryApplicationQueryBatchResourceDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};

mod execution;

/// A homogeneous, ordered batch of existing declared intents. All items use
/// the retained data observation, but retain independent ordinary scope and
/// disclosure admission. Duplicate scopes remain separate, charged entries.
///
/// `limits` is required before execution. The common loan covers root/tree
/// read work and result/source custody. Authorization and preparation retain
/// their existing per-item bounds and are not claimed as aggregate-bounded.
///
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::WorthQueryApplicationQueryBatchRequest;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_query::ApplicationQueryIntent};
/// fn unbounded<S: ApplicationSchema, I: ApplicationQueryIntent<S>>(request: WorthQueryApplicationQueryBatchRequest<'_, '_, '_, S, I>) {
///     request.execute();
/// }
/// ```
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

/// A batch with mandatory explicit read/custody bounds. This is a bounded
/// builder, not an admitted query or scope capability.
pub struct WorthQueryApplicationBoundedQueryBatchRequest<
    'application,
    'principal,
    'scope,
    Schema,
    Intent,
> {
    request:
        WorthQueryApplicationQueryBatchRequest<'application, 'principal, 'scope, Schema, Intent>,
    limits: WorthQueryApplicationQueryBatchLimits,
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

    pub fn limits(
        self,
        limits: WorthQueryApplicationQueryBatchLimits,
    ) -> WorthQueryApplicationBoundedQueryBatchRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
    > {
        WorthQueryApplicationBoundedQueryBatchRequest {
            request: self,
            limits,
        }
    }
}

impl<
        'application,
        'principal,
        'scope,
        Schema: ApplicationSchema,
        Intent: ApplicationQueryIntent<Schema>,
    >
    WorthQueryApplicationBoundedQueryBatchRequest<'application, 'principal, 'scope, Schema, Intent>
{
    pub fn execute(self) -> Result<
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
        execution::execute(self)
    }
}
