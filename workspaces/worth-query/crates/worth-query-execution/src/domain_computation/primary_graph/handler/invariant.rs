use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryOperationScopeBinding,
};
use super::DecisionContextUse;

mod predecode_admission;

/// Why a mutation handler stopped before finishing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandlerInterruption {
    /// The request was cancelled.
    Cancelled,
    /// The request reached its deadline.
    DeadlineExceeded,
}

impl std::fmt::Display for HandlerInterruption {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Cancelled => "the request was cancelled",
            Self::DeadlineExceeded => "the request reached its deadline",
        })
    }
}

impl std::error::Error for HandlerInterruption {}

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
    pub(super) reader: &'borrow mut WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        Schema,
        Binding::Operation,
    >,
    scope: &'borrow WorthQueryInvariantEntityIdentity<
        Schema,
        <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    >,
    principal_identity: &'borrow Binding::PrincipalIdentity,
    operation_scope_binding: &'borrow WorthQueryOperationScopeBinding,
    identities: &'borrow ApplicationMutationIdentities<'borrow, Schema, Binding>,
    request:
        &'borrow worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    context_use: &'borrow Cell<DecisionContextUse>,
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
        principal_identity: &'borrow Binding::PrincipalIdentity,
        operation_scope_binding: &'borrow WorthQueryOperationScopeBinding,
        identities: &'borrow ApplicationMutationIdentities<'borrow, Schema, Binding>,
        request: &'borrow worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        context_use: &'borrow Cell<DecisionContextUse>,
    ) -> Self {
        Self {
            reader,
            scope,
            principal_identity,
            operation_scope_binding,
            identities,
            request,
            context_use,
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
        self.context_use.set(self.context_use.get().opaque_reader());
        self.reader
    }

    pub fn scope(
        &self,
    ) -> &WorthQueryInvariantEntityIdentity<
        Schema,
        <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    > {
        self.context_use.set(self.context_use.get().scope());
        self.scope
    }

    pub fn idempotency_key(&self) -> &Binding::IdempotencyKey {
        self.context_use.set(self.context_use.get().key());
        self.identities.idempotency_key()
    }

    /// Identity of the request's idempotency key, encoded once for the whole
    /// request and scoped to the binding's key namespace.
    pub fn key_identity(&self) -> &[u8; 32] {
        self.context_use.set(self.context_use.get().key());
        self.identities.key_identity()
    }

    /// Identity of the request's input, encoded once for the whole request.
    /// A handler that derives a domain identity from the input uses this one,
    /// never a second encoding.
    pub fn input_identity(&self) -> &[u8; 32] {
        self.identities.input_identity()
    }

    /// Resolved application principal identity for this admitted operation.
    /// This is decision data; the admission proof remains the authority.
    pub fn principal_identity(&self) -> &Binding::PrincipalIdentity {
        self.context_use.set(self.context_use.get().principal());
        self.principal_identity
    }

    /// Descriptive installed operation/principal/scope affinity used for
    /// deterministic domain identities. This value carries no authority.
    pub fn operation_scope_binding(&self) -> &WorthQueryOperationScopeBinding {
        self.context_use
            .set(self.context_use.get().scope().principal());
        self.operation_scope_binding
    }

    pub fn checkpoint(&self) -> Result<(), HandlerInterruption> {
        self.context_use
            .set(self.context_use.get().request_context());
        self.request.interruption().map_or(Ok(()), |interruption| {
            Err(HandlerInterruption::from(interruption))
        })
    }

    pub fn managed_computation_execution(
        &self,
    ) -> crate::domain_computation::primary_graph::WorthQueryManagedComputationExecution<'_> {
        self.context_use
            .set(self.context_use.get().managed_computation());
        crate::domain_computation::primary_graph::WorthQueryManagedComputationExecution::new(
            self.request,
        )
    }
}
use std::cell::Cell;
