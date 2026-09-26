use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::{
    application_installation::WorthQueryWorkflowVocabulary,
    workflow_instance_start::{
        PreparedWorkflowInstanceStart, PublishedWorkflowDefinitionRef,
        WorkflowInstancePreparationDenial, WorkflowInstanceStartOutcome,
        WorthQueryWorkflowInstanceStartAdapter,
    },
};
use worth_query_installation::facade::ApplicationSchema;

use crate::application_entry::{
    mutation::{authorization, WorthQueryApplicationMutationRequestWithIdempotency},
    WorthQueryApplicationRequestMutationDenial,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowInstanceStartPreparationDenialKind {
    RuntimeMismatch,
    RequestAdmission,
    InstancePreparation,
}

#[derive(Debug)]
pub enum WorthQueryWorkflowInstanceStartPreparationDenial {
    RuntimeMismatch,
    RequestAdmission(WorthQueryApplicationRequestMutationDenial),
    InstancePreparation(WorkflowInstancePreparationDenial),
}

impl WorthQueryWorkflowInstanceStartPreparationDenial {
    pub const fn kind(&self) -> WorthQueryWorkflowInstanceStartPreparationDenialKind {
        match self {
            Self::RuntimeMismatch => {
                WorthQueryWorkflowInstanceStartPreparationDenialKind::RuntimeMismatch
            }
            Self::RequestAdmission(_) => {
                WorthQueryWorkflowInstanceStartPreparationDenialKind::RequestAdmission
            }
            Self::InstancePreparation(_) => {
                WorthQueryWorkflowInstanceStartPreparationDenialKind::InstancePreparation
            }
        }
    }
}

impl std::fmt::Display for WorthQueryWorkflowInstanceStartPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "workflow instance-start preparation denied: {:?}",
            self.kind()
        )
    }
}

impl std::error::Error for WorthQueryWorkflowInstanceStartPreparationDenial {}

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
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    MutationInput<Schema, Intent>: Clone
        + ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
{
    pub fn prepare_workflow_instance_start<'workflow, Spec, Program: 'workflow>(
        mut self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec, Program>>,
        definition: PublishedWorkflowDefinitionRef,
    ) -> Result<
        WorthQueryWorkflowInstanceStartRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowInstanceStartPreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let workflow = workflow.into();
        let application = self.application_runtime();
        if !std::ptr::eq(application, workflow.runtime()) {
            return Err(WorthQueryWorkflowInstanceStartPreparationDenial::RuntimeMismatch);
        }
        let selected = application
            .on_branch(self.product_branch())
            .select()
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)
            .map_err(WorthQueryWorkflowInstanceStartPreparationDenial::RequestAdmission)?;
        let mutation = authorization::prepare_capability_selected(&mut self, &selected)
            .map_err(WorthQueryWorkflowInstanceStartPreparationDenial::RequestAdmission)?;
        let prepared = WorthQueryWorkflowInstanceStartAdapter::prepare::<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
            Spec,
            Program,
        >(
            &selected,
            workflow.workflow_spec(),
            definition,
            *mutation.idempotency.key_identity(),
            mutation.admission,
        )
        .map_err(WorthQueryWorkflowInstanceStartPreparationDenial::InstancePreparation)?;
        Ok(WorthQueryWorkflowInstanceStartRequest {
            application,
            prepared,
            idempotency: mutation.idempotency,
        })
    }
}

pub struct WorthQueryWorkflowInstanceStartRequest<'application, Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
{
    application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    prepared: PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
    idempotency:
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding,
}

impl<Schema, Operation, Input, Scope>
    WorthQueryWorkflowInstanceStartRequest<'_, Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn execute(self) -> WorkflowInstanceStartOutcome {
        WorthQueryWorkflowInstanceStartAdapter::compare_and_commit(
            self.application,
            self.prepared,
            self.idempotency,
        )
    }
}
