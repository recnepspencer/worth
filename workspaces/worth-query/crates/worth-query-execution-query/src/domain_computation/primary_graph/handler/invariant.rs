use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryOperationScopeBinding,
};
use super::DecisionContextUse;

mod interruption;
mod managed_index;
mod operation_body;
mod predecessor;
mod predecode_admission;

pub use worth_query_execution_lineage::facade::primary_graph::handler::HandlerInterruption;

/// Exact admitted decision projection and its already-resolved operation scope.
pub struct DecisionReader<'borrow, 'reader, 'runtime, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(crate) reader: &'borrow mut WorthQueryApplicationOperationInvariantProjectionReader<
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
    execution: &'borrow QueryRequestExecution<'borrow>,
    pub(crate) context_use: &'borrow Cell<DecisionContextUse>,
    pub(crate) conditional_definitions: super::ConditionalDefinitionAdmission<'borrow, Schema>,
    interruption: &'borrow Cell<Option<HandlerInterruption>>,
    pub(crate) source_query: Option<
        &'borrow crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess<
            'borrow,
            Schema,
        >,
    >,
}

impl<'borrow, 'reader, 'runtime, Schema, Binding>
    DecisionReader<'borrow, 'reader, 'runtime, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
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
        execution: &'borrow QueryRequestExecution<'borrow>,
        context_use: &'borrow Cell<DecisionContextUse>,
        interruption: &'borrow Cell<Option<HandlerInterruption>>,
        conditional_definitions: super::ConditionalDefinitionAdmission<'borrow, Schema>,
        source_query: Option<
            &'borrow crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess<
                'borrow,
                Schema,
            >,
        >,
    ) -> Self {
        Self {
            reader,
            scope,
            principal_identity,
            operation_scope_binding,
            identities,
            execution,
            context_use,
            interruption,
            conditional_definitions,
            source_query,
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

    /// Zero-based authored-stage position; absent outside an edit envelope.
    pub fn stage_ordinal(&self) -> Option<usize> {
        // Ordinal-sensitive decisions must not reuse an ordinary decision that
        // retained only the domain key. This existing posture disables reuse
        // without weakening the tracked reader or inventing a key dependency.
        self.context_use.set(self.context_use.get().stage_ordinal());
        self.identities
            .edit_stage_identity()
            .map(|stage| stage.ordinal())
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

    /// Polls real request interruption without making it decision data.
    /// Preparation retains the first failure and refuses the returned plan,
    /// even when the handler catches or ignores this result.
    pub fn checkpoint(&self) -> Result<(), HandlerInterruption> {
        interruption::checkpoint(self.execution, self.interruption)
    }

    /// The operation's reader, for a partitioned computation this handler
    /// runs to read its input through.
    pub fn operation_reader(
        &mut self,
    ) -> &mut WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        Schema,
        Binding::Operation,
    > {
        self.reader
    }

    /// The request's execution, for a partitioned computation this handler
    /// runs.
    pub const fn execution(&self) -> &'borrow QueryRequestExecution<'borrow> {
        self.execution
    }

    #[doc(hidden)]
    pub fn mark_opaque_reader(&self) {
        self.context_use.set(self.context_use.get().opaque_reader());
    }

    #[doc(hidden)]
    pub fn conditional_definitions(
        &self,
    ) -> &super::ConditionalDefinitionAdmission<'borrow, Schema> {
        &self.conditional_definitions
    }
}
use std::cell::Cell;

use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;

mod court_doors_for_test;
