use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
    ApplicationMutationSourceExpectation, NoApplicationMutationSource,
};
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;

#[doc(hidden)]
pub struct WorthQueryMutationSourceUnprepared;

#[doc(hidden)]
pub struct WorthQueryMutationSourcePrepared;

pub(super) enum WorthQueryMutationExpectedSource<Query> {
    Row(worth_query_execution::facade::primary_graph::WorthQueryObservedSource<Query>),
    ResultSet(worth_query_execution::facade::primary_graph::WorthQueryObservedResultSet<Query>),
}

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
    pub(super) key: &'key <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    pub(super) workflow_transition_identity: Option<[u8; 32]>,
    pub(super) workflow_authority: Option<
        std::sync::Arc<
            worth_query_execution::facade::workflow_advance::WorkflowOperationAuthoritySlot,
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
            workflow_transition_identity: None,
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
        authority: std::sync::Arc<
            worth_query_execution::facade::workflow_advance::WorkflowOperationAuthoritySlot,
        >,
    ) -> Self {
        self.workflow_transition_identity = Some(identity);
        self.workflow_authority = Some(authority);
        self
    }

    pub(in crate::application_entry) fn bind_workflow_recovery_transition(
        mut self,
        identity: [u8; 32],
    ) -> Self {
        self.workflow_transition_identity = Some(identity);
        self
    }

    pub(in crate::application_entry) fn input_identity(&self) -> [u8; 32] {
        Intent::Binding::input_identity(self.request.intent.input())
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
