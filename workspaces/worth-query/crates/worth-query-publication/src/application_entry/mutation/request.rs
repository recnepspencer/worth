mod birth;
mod input_identity;
mod source;
mod succession;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationBinding, ApplicationMutationIdentities,
    ApplicationMutationIdentityDenial, ApplicationMutationIntent, ApplicationMutationScopeBinding,
    ApplicationMutationSourceExpectation,
};
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;

use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

#[doc(hidden)]
pub struct WorthQueryMutationSourceUnprepared;

/// Marks a mutation request whose source expectation is settled.
#[doc(hidden)]
pub struct WorthQueryMutationSourcePrepared;

pub(in crate::application_entry) enum WorthQueryMutationExpectedSource<Query> {
    Row(worth_query_execution::facade::primary_graph::WorthQueryObservedSource<Query>),
    ResultSet(worth_query_execution::facade::primary_graph::WorthQueryObservedResultSet<Query>),
}

impl<Query> Clone for WorthQueryMutationExpectedSource<Query> {
    fn clone(&self) -> Self {
        match self {
            Self::Row(source) => Self::Row(source.clone()),
            Self::ResultSet(source) => Self::ResultSet(source.clone()),
        }
    }
}

/// A mutation request under construction. State its source expectation and preconditions,
/// then bind an idempotency key.
pub struct WorthQueryApplicationMutationRequest<
    'application,
    'principal,
    'scope,
    Schema,
    Intent,
    SourcePreparation = WorthQueryMutationSourceUnprepared,
>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(in crate::application_entry) application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(in crate::application_entry) principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(in crate::application_entry) scope: &'scope WorthQueryRequestScope,
    pub(in crate::application_entry) branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    pub(in crate::application_entry) intent: Intent,
    pub(in crate::application_entry) producer_round_limits: super::ApplicationProducerRoundLimits,
    pub(in crate::application_entry) preconditions: TypedMutationPreconditions<
        Schema,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
        <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    >,
    pub(in crate::application_entry) source: Option<
        WorthQueryMutationExpectedSource<
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
    >,
    pub(in crate::application_entry) source_preparation: std::marker::PhantomData<SourcePreparation>,
}

/// A mutation request bound to its idempotency key, ready to execute.
pub struct WorthQueryApplicationMutationRequestWithIdempotency<
    'application,
    'principal,
    'scope,
    'key,
    Schema,
    Intent,
    SourcePreparation = WorthQueryMutationSourceUnprepared,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(in crate::application_entry) request: WorthQueryApplicationMutationRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        SourcePreparation,
    >,
    pub(in crate::application_entry) compiled_action: Option<&'scope worth_query_declaration::facade::application_program::ApplicationActionDeclaration>,
    pub(in crate::application_entry) history_constraint: Option<&'scope worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryOperationConstraint>,
    pub(in crate::application_entry) borrowed_preconditions: Option<&'scope TypedMutationPreconditions<
        Schema,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
        <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    >>,
    pub(in crate::application_entry) operation_input: Option<&'scope Result<ApplicationEncodedInput<<Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding>, worth_query_declaration::facade::application_schema::ApplicationValueEncodeDenial>>,
    pub(super) unstaged: Option<super::staged::UnstagedMutation>,
    pub(in crate::application_entry) key: &'key <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    pub(in crate::application_entry) workflow_actor_context: Option<worth_query_execution::publication_boundary::workflow_advance::WorthQueryWorkflowActorContext>,
    pub(in crate::application_entry) workflow_transition_identity: Option<[u8; 32]>,
    pub(in crate::application_entry) workflow_captured_input:
        Option<worth_query_execution::publication_boundary::workflow_proposal::WorkflowCapturedInputContract>,
    pub(super) workflow_birth_identity: Option<[u8; 32]>,
    pub(super) workflow_succession_identity: Option<[u8; 32]>,
    pub(super) workflow_succession_authority: Option<std::sync::Arc<worth_query_execution::publication_boundary::workflow_succession::WorkflowSuccessionAuthoritySlot>>,
    pub(super) workflow_birth_authority: Option<std::sync::Arc<worth_query_execution::publication_boundary::workflow_birth::WorkflowBirthAuthoritySlot>>,
    /// The input, encoded once when the request bound its workflow
    /// requirement, so the entry point encodes only the key beside it.
    pub(in crate::application_entry) workflow_input: Option<
        ApplicationEncodedInput<<Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding>,
    >,
    pub(in crate::application_entry) workflow_authority: Option<
        std::sync::Arc<
            worth_query_execution::publication_boundary::workflow_advance::WorkflowOperationAuthoritySlot,
        >,
    >,
}

impl<'application, 'principal, 'scope, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(in crate::application_entry) const fn application_runtime(
        &self,
    ) -> &'application WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.application
    }

    pub(in crate::application_entry) const fn product_branch(
        &self,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.branch
    }

    pub(in crate::application_entry) const fn request_scope(
        &self,
    ) -> &'scope WorthQueryRequestScope {
        self.scope
    }

    /// Bounds real synchronous producer starts within this edit attempt.
    pub fn producer_round_limits(mut self, limits: super::ApplicationProducerRoundLimits) -> Self {
        self.producer_round_limits = limits;
        self
    }

    pub fn preconditions(
        mut self,
        preconditions: TypedMutationPreconditions<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
    ) -> Self {
        self.preconditions = preconditions;
        self
    }

    pub fn idempotency<'key>(
        self,
        key: &'key <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    ) -> WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    > {
        WorthQueryApplicationMutationRequestWithIdempotency {
            request: self,
            key,
            compiled_action: None,
            history_constraint: None,
            borrowed_preconditions: None,
            operation_input: None,
            unstaged: Some(super::staged::UnstagedMutation),
            workflow_transition_identity: None,
            workflow_captured_input: None,
            workflow_birth_identity: None,
            workflow_succession_identity: None,
            workflow_succession_authority: None,
            workflow_birth_authority: None,
            workflow_input: None,
            workflow_authority: None,
            workflow_actor_context: None,
        }
    }
}

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(in crate::application_entry) fn bind_workflow_transition(
        mut self,
        identity: [u8; 32],
        input: ApplicationEncodedInput<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
        >,
        authority: std::sync::Arc<
            worth_query_execution::publication_boundary::workflow_advance::WorkflowOperationAuthoritySlot,
        >,
    ) -> Self {
        self.workflow_transition_identity = Some(identity);
        self.workflow_input = Some(input);
        self.workflow_authority = Some(authority);
        self
    }

    pub(in crate::application_entry) fn bind_workflow_recovery_transition(
        mut self,
        identity: [u8; 32],
        input: ApplicationEncodedInput<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
        >,
    ) -> Self {
        self.workflow_transition_identity = Some(identity);
        self.workflow_input = Some(input);
        self
    }

    pub(in crate::application_entry) const fn workflow_transition_identity(
        &self,
    ) -> Option<[u8; 32]> {
        self.workflow_transition_identity
    }

    pub(in crate::application_entry) const fn application_runtime(
        &self,
    ) -> &'application WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.request.application_runtime()
    }

    pub(in crate::application_entry) const fn product_branch(
        &self,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.request.product_branch()
    }

    pub(in crate::application_entry) const fn authenticated_principal(
        &self,
    ) -> &'principal WorthQueryAuthenticatedExternalPrincipal<Schema> {
        self.request.principal
    }

    pub(in crate::application_entry) const fn request_scope(
        &self,
    ) -> &'scope WorthQueryRequestScope {
        self.request.scope
    }
}
