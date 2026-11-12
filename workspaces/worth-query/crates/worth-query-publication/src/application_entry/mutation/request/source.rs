//! Source-expectation progression preserves the complete entry policy.
use super::{
    WorthQueryApplicationMutationRequest, WorthQueryMutationExpectedSource,
    WorthQueryMutationSourcePrepared,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationSourceExpectation,
    NoApplicationMutationSource,
};
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
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
            producer_round_limits: Default::default(),
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
            producer_round_limits: self.producer_round_limits,
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
            producer_round_limits: self.producer_round_limits,
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
            producer_round_limits: self.producer_round_limits,
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
        .producer_round_limits(self.producer_round_limits)
    }
}
