//! Capability-governed and approved-elevation typed live subscriptions.
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
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
use worth_query_publication::facade::application_entry::WorthQueryApplicationLiveOpenRequestDenial;

use super::live_subscription::WorthQueryTypedLiveSubscription;
use super::{QueryBinding, QueryMarker, QueryRow, QueryScope, WorthQueryHostSession};

impl<'home, Schema, Program> WorthQueryHostSession<'home, Schema, Program>
where
    Schema: ApplicationSchema,
{
    /// Opens a capability-governed typed whole-result live subscription.
    pub fn subscribe_governed<Intent, Capability, Operation, Input>(
        &self,
        intent: Intent,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryTypedLiveSubscription<'home, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
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
        let principal = self.call_principal();
        let request = self.request_as(&principal, &scope);
        let query_req = request.query(intent);
        let query_req = match self.settings.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        let limits = self.resolved_live_limits::<Intent>();
        self.with_funded_advancement(&scope, |phase| {
            query_req.subscribe_whole_result_governed_in_advancement(
                &phase, capability, operation, input, limits,
            )
        })
        .map_err(WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest)?
        .map(WorthQueryTypedLiveSubscription::from_opened)
    }

    /// Opens an approved-elevation typed whole-result live subscription.
    pub fn subscribe_approved<Intent, Capability, Operation, Input>(
        &self,
        intent: Intent,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryTypedLiveSubscription<'home, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
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
        let principal = self.call_principal();
        let request = self.request_as(&principal, &scope);
        let query_req = request.query(intent);
        let query_req = match self.settings.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        let limits = self.resolved_live_limits::<Intent>();
        self.with_funded_advancement(&scope, |phase| {
            query_req.subscribe_whole_result_approved_in_advancement(
                &phase, approved, capability, operation, input, limits,
            )
        })
        .map_err(WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest)?
        .map(WorthQueryTypedLiveSubscription::from_opened)
    }
}
