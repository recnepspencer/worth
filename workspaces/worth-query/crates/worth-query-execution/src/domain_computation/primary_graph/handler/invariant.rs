use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandlerInterruption {
    Cancelled,
    DeadlineExceeded,
}

impl From<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    for HandlerInterruption
{
    fn from(
        interruption: worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption,
    ) -> Self {
        match interruption {
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => Self::Cancelled,
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => Self::DeadlineExceeded,
        }
    }
}

/// Exact admitted decision projection and its already-resolved operation scope.
pub struct DecisionReader<'borrow, 'reader, 'runtime, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    reader: &'borrow mut WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        Schema,
        Binding::Operation,
    >,
    scope: &'borrow WorthQueryInvariantEntityIdentity<
        Schema,
        <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    >,
    idempotency_key: &'borrow Binding::IdempotencyKey,
    request:
        &'borrow worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
}

impl<'borrow, 'reader, 'runtime, Schema, Binding>
    DecisionReader<'borrow, 'reader, 'runtime, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(in crate::domain_computation::primary_graph) fn new(
        reader: &'borrow mut WorthQueryApplicationOperationInvariantProjectionReader<
            'reader,
            'runtime,
            Schema,
            Binding::Operation,
        >,
        scope: &'borrow WorthQueryInvariantEntityIdentity<
            Schema,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency_key: &'borrow Binding::IdempotencyKey,
        request: &'borrow worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Self {
        Self {
            reader,
            scope,
            idempotency_key,
            request,
        }
    }

    pub fn reader(
        &mut self,
    ) -> &mut WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        Schema,
        Binding::Operation,
    > {
        self.reader
    }

    pub fn scope(
        &self,
    ) -> &WorthQueryInvariantEntityIdentity<
        Schema,
        <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    > {
        self.scope
    }

    pub fn idempotency_key(&self) -> &Binding::IdempotencyKey {
        self.idempotency_key
    }

    pub fn checkpoint(&self) -> Result<(), HandlerInterruption> {
        self.request.interruption().map_or(Ok(()), |interruption| {
            Err(HandlerInterruption::from(interruption))
        })
    }
}
