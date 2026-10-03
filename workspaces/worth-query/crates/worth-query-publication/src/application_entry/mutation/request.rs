use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationBinding, ApplicationMutationIdentities,
    ApplicationMutationIdentityDenial, ApplicationMutationIntent, ApplicationMutationScopeBinding,
    ApplicationMutationSourceExpectation, NoApplicationMutationSource,
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

pub(super) enum WorthQueryMutationExpectedSource<Query> {
    Row(worth_query_execution::facade::primary_graph::WorthQueryObservedSource<Query>),
    ResultSet(worth_query_execution::facade::primary_graph::WorthQueryObservedResultSet<Query>),
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
    pub(super) application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: &'scope WorthQueryRequestScope,
    pub(super) branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    pub(super) intent: Intent,
    pub(super) preconditions: TypedMutationPreconditions<
        Schema,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
        <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    >,
    pub(super) source: Option<
        WorthQueryMutationExpectedSource<
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
    >,
    source_preparation: std::marker::PhantomData<SourcePreparation>,
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
    pub(super) request: WorthQueryApplicationMutationRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        SourcePreparation,
    >,
    pub(super) unstaged: Option<super::staged::UnstagedMutation>,
    pub(super) key: &'key <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    pub(super) workflow_transition_identity: Option<[u8; 32]>,
    /// The input, encoded once when the request bound its workflow
    /// requirement, so the entry point encodes only the key beside it.
    pub(super) workflow_input: Option<
        ApplicationEncodedInput<<Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding>,
    >,
    pub(super) workflow_authority: Option<
        std::sync::Arc<
            worth_query_execution::publication_boundary::workflow_advance::WorkflowOperationAuthoritySlot,
        >,
    >,
}

impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationMutationRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(in crate::application_entry) fn new(
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
            intent,
            preconditions: TypedMutationPreconditions::default(),
            source: None,
            source_preparation: std::marker::PhantomData,
        }
    }

    pub fn expect_source(
        self,
        source: worth_query_execution::facade::primary_graph::WorthQueryObservedSource<
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
    ) -> WorthQueryApplicationMutationRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        WorthQueryMutationSourcePrepared,
    > {
        WorthQueryApplicationMutationRequest {
            application: self.application,
            principal: self.principal,
            scope: self.scope,
            branch: self.branch,
            intent: self.intent,
            preconditions: self.preconditions,
            source: Some(WorthQueryMutationExpectedSource::Row(source)),
            source_preparation: std::marker::PhantomData,
        }
    }

    pub fn expect_result_set(
        self,
        source: worth_query_execution::facade::primary_graph::WorthQueryObservedResultSet<
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
    ) -> WorthQueryApplicationMutationRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        WorthQueryMutationSourcePrepared,
    > {
        WorthQueryApplicationMutationRequest {
            application: self.application,
            principal: self.principal,
            scope: self.scope,
            branch: self.branch,
            intent: self.intent,
            preconditions: self.preconditions,
            source: Some(WorthQueryMutationExpectedSource::ResultSet(source)),
            source_preparation: std::marker::PhantomData,
        }
    }

    pub fn without_source(
        self,
    ) -> WorthQueryApplicationMutationRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        WorthQueryMutationSourcePrepared,
    >
    where
        Intent::Binding:
            ApplicationMutationBinding<Schema, SourceExpectation = NoApplicationMutationSource>,
    {
        WorthQueryApplicationMutationRequest {
            application: self.application,
            principal: self.principal,
            scope: self.scope,
            branch: self.branch,
            intent: self.intent,
            preconditions: self.preconditions,
            source: None,
            source_preparation: std::marker::PhantomData,
        }
    }

    pub(in crate::application_entry) fn repeat_for_workflow_run(&self) -> Self
    where
        Intent: Clone,
        Intent::Binding:
            ApplicationMutationBinding<Schema, SourceExpectation = NoApplicationMutationSource>,
    {
        Self::new(
            self.application,
            self.principal,
            self.scope,
            self.branch,
            self.intent.clone(),
        )
        .preconditions(self.preconditions.clone())
    }
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
            unstaged: Some(super::staged::UnstagedMutation),
            workflow_transition_identity: None,
            workflow_input: None,
            workflow_authority: None,
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

    /// The request's key and input with the identities encoded from exactly them.
    ///
    /// Each entry point calls this once and passes the result through preparation,
    /// execution, and the idempotency binding; the identities borrow the key and
    /// the input this request owns, so they cannot be stored beside them. A
    /// request bound to a workflow requirement already encoded its input when it
    /// bound, so only the key is encoded here.
    pub(in crate::application_entry) fn identities(
        &self,
    ) -> Result<
        ApplicationMutationIdentities<'_, Schema, Intent::Binding>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        match &self.workflow_input {
            Some(input) => ApplicationMutationIdentities::with_encoded_input(self.key, input),
            None => ApplicationMutationIdentities::encode(self.key, self.request.intent.input()),
        }
        .map_err(WorthQueryApplicationRequestMutationDenial::Identity)
    }

    /// Encodes the request's input once, for binding it to a workflow
    /// requirement; the entry point later reuses it through `identities`.
    pub(in crate::application_entry) fn encode_input(
        &self,
    ) -> Result<
        ApplicationEncodedInput<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
        >,
        ApplicationMutationIdentityDenial,
    >
    where
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone,
    {
        ApplicationEncodedInput::encode(self.request.intent.input().clone())
            .map_err(ApplicationMutationIdentityDenial::Input)
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
