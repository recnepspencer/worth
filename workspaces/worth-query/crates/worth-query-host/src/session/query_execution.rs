//! One-shot query execution methods on [`WorthQueryHostSession`], enforcing
//! the session's configured [`ExecutionRequestPolicy`].
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_program::ApplicationProgramDefinition,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApprovedElevation,
};
use worth_query_publication::facade::{
    application_entry::WorthQueryApplicationRequestQueryDenial,
    domain_computation::WorthQueryPublishedApplicationResult,
};

use super::{QueryBinding, QueryMarker, QueryRow, QueryScope, WorthQueryHostSession};

impl<Schema, Program> WorthQueryHostSession<'_, Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// Executes a one-shot query and returns its typed projected rows directly.
    pub fn query<Intent>(
        &self,
        intent: Intent,
    ) -> Result<Vec<QueryRow<Schema, Intent>>, WorthQueryApplicationRequestQueryDenial>
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
        self.query_result(intent)
            .map(|published| published.into_rows())
    }

    /// Executes a one-shot query and retains the full published result (rows,
    /// publication receipt, and observed sources).
    pub fn query_result<Intent>(
        &self,
        intent: Intent,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Intent>, QueryRow<Schema, Intent>>,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        let query_req = request.query(intent);
        let query_req = match self.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        self.with_funded_advancement(&scope, |phase| query_req.execute_in_advancement(&phase))
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }

    /// Executes a capability-governed one-shot query and returns its typed rows.
    pub fn query_governed<Intent, Capability, Operation, Input>(
        &self,
        intent: Intent,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<Vec<QueryRow<Schema, Intent>>, WorthQueryApplicationRequestQueryDenial>
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = QueryScope<Schema, Intent>>
            + 'static,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        let query_req = request.query(intent);
        let query_req = match self.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        self.with_funded_advancement(&scope, |phase| {
            query_req.execute_governed_in_advancement(&phase, capability, operation, input)
        })
        .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
        .map(|published| published.into_rows())
    }

    /// Executes an approved-elevation one-shot query and returns its typed rows.
    pub fn query_approved<Intent, Capability, Operation, Input>(
        &self,
        intent: Intent,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<Vec<QueryRow<Schema, Intent>>, WorthQueryApplicationRequestQueryDenial>
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = QueryScope<Schema, Intent>>
            + 'static,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        let query_req = request.query(intent);
        let query_req = match self.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        self.with_funded_advancement(&scope, |phase| {
            query_req
                .execute_approved_in_advancement(&phase, approved, capability, operation, input)
        })
        .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
        .map(|published| published.into_rows())
    }
}
