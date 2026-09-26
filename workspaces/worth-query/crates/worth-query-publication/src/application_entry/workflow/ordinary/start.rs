use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution, NoApplicationMutationSource,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::{
    application_installation::WorthQueryWorkflowVocabulary,
    workflow_instance_start::{PublishedWorkflowDefinitionRef, WorkflowInstanceStartOutcome},
};
use worth_query_installation::facade::ApplicationSchema;

use crate::application_entry::{
    mutation::{
        WorthQueryApplicationMutationRequest, WorthQueryApplicationMutationRequestWithIdempotency,
        WorthQueryMutationSourcePrepared,
    },
    WorthQueryWorkflowInstanceStartPreparationDenial,
};

type IntentBinding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope;
type MutationOperation<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Operation;
type MutationInput<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Input;

pub struct WorthQueryOrdinaryWorkflowStart<
    'application,
    'principal,
    'scope,
    Schema,
    Intent,
    Spec,
    Program,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    request: WorthQueryApplicationMutationRequest<'application, 'principal, 'scope, Schema, Intent>,
    workflow: WorthQueryWorkflowVocabulary<'application, Schema, Spec, Program>,
    definition: PublishedWorkflowDefinitionRef,
}

pub struct WorthQueryOrdinaryWorkflowStartWithIdempotency<
    'application,
    'principal,
    'scope,
    'key,
    Schema,
    Intent,
    Spec,
    Program,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    request: WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        WorthQueryMutationSourcePrepared,
    >,
    workflow: WorthQueryWorkflowVocabulary<'application, Schema, Spec, Program>,
    definition: PublishedWorkflowDefinitionRef,
}

impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationMutationRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    /// Subjects and intended capability remain in the caller's typed mutation intent.
    pub fn start_workflow<Spec, Program>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec, Program>>,
        definition: PublishedWorkflowDefinitionRef,
    ) -> WorthQueryOrdinaryWorkflowStart<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        Spec,
        Program,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let workflow = workflow.into();
        WorthQueryOrdinaryWorkflowStart {
            request: self,
            workflow,
            definition,
        }
    }
}

impl<'application, 'principal, 'scope, Schema, Intent, Spec, Program>
    WorthQueryOrdinaryWorkflowStart<'application, 'principal, 'scope, Schema, Intent, Spec, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    IntentBinding<Schema, Intent>:
        ApplicationMutationBinding<Schema, SourceExpectation = NoApplicationMutationSource>,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn idempotency<'key>(
        self,
        key: &'key <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    ) -> WorthQueryOrdinaryWorkflowStartWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        Spec,
        Program,
    > {
        WorthQueryOrdinaryWorkflowStartWithIdempotency {
            request: self.request.without_source().idempotency(key),
            workflow: self.workflow,
            definition: self.definition,
        }
    }
}

impl<Schema, Intent, Spec, Program>
    WorthQueryOrdinaryWorkflowStartWithIdempotency<'_, '_, '_, '_, Schema, Intent, Spec, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    MutationOperation<Schema, Intent>: 'static,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    MutationInput<Schema, Intent>: Clone + Send + Sync + 'static
        + ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn execute(self) -> Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial> {
        let prepared = self.request.prepare_workflow_instance_start(self.workflow, self.definition)?;
        Ok(prepared.execute())
    }
}
