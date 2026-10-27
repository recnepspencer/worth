//! One-shot opt-in; no live subscription or continuation authority is exposed.
use super::{WorthQueryApplicationQueryRequest, WorthQueryApplicationRequestQueryDenial};
use crate::domain_computation::WorthQueryPublishedApplicationResult;
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
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApprovedElevation,
};

type Binding<S, I> = <I as ApplicationQueryIntent<S>>::Binding;
type Query<S, I> = <Binding<S, I> as ApplicationQueryBinding<S>>::Query;
type Value<S, I> = <<Binding<S, I> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Scope<S, I> =
    <<Binding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<
        S,
    >>::Scope;

/// Explicit one-shot retention of the original Native disclosed projection.
/// The ordinary original query still owns admission, decoding and completion.
/// The retained backing is descriptive and grants no later inspection access.
pub struct WorthQueryApplicationProjectionRetentionRequest<
    'application,
    'principal,
    'scope,
    Schema,
    Intent,
> {
    request: WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent>,
}

impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationProjectionRetentionRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
{
    pub(super) fn new(
        request: WorthQueryApplicationQueryRequest<
            'application,
            'principal,
            'scope,
            Schema,
            Intent,
        >,
    ) -> Self {
        Self { request }
    }

    pub fn limit_results(mut self, maximum_results: NonZeroUsize) -> Self {
        self.request = self.request.limit_results(maximum_results);
        self
    }
}

impl<Schema, Intent> WorthQueryApplicationProjectionRetentionRequest<'_, '_, '_, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    Value<Schema, Intent>: WorthQueryApplicationProjection<Schema, Query<Schema, Intent>>,
{
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<Query<Schema, Intent>, Value<Schema, Intent>>,
        WorthQueryApplicationRequestQueryDenial,
    > {
        self.request.execute()
    }

    pub fn execute_governed<Capability, Operation, Input>(
        self,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryPublishedApplicationResult<Query<Schema, Intent>, Value<Schema, Intent>>,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.request.execute_governed(capability, operation, input)
    }

    pub fn execute_approved<Capability, Operation, Input>(
        self,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryPublishedApplicationResult<Query<Schema, Intent>, Value<Schema, Intent>>,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.request
            .execute_approved(approved, capability, operation, input)
    }
}
