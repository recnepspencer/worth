use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeResolution,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::{
    application_installation::WorthQueryWorkflowVocabulary,
    primary_graph::{
        WorthQueryApplicationIdempotencyBinding, WorthQueryPrimaryGraphApplicationRuntime,
    },
    workflow_definition_retirement::{
        PreparedWorkflowDefinitionRetirement, PublishedWorkflowDefinitionRef,
        WorkflowDefinitionPreparationDenial, WorkflowDefinitionRetirementOutcome,
        WorthQueryWorkflowDefinitionRetirementAdapter,
    },
};
use worth_query_installation::facade::ApplicationSchema;

use super::{IntentBinding, MutationInput, MutationOperation, MutationScope};
use crate::application_entry::{
    mutation::{authorization, WorthQueryApplicationMutationRequestWithIdempotency},
    WorthQueryApplicationRequestMutationDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowDefinitionRetirementPreparationDenialKind {
    RuntimeMismatch,
    RequestAdmission,
    DefinitionPreparation,
}

#[derive(Debug)]
pub enum WorthQueryWorkflowDefinitionRetirementPreparationDenial {
    RuntimeMismatch,
    RequestAdmission(WorthQueryApplicationRequestMutationDenial),
    DefinitionPreparation(WorkflowDefinitionPreparationDenial),
}

type PreparationDenial = WorthQueryWorkflowDefinitionRetirementPreparationDenial;

impl WorthQueryWorkflowDefinitionRetirementPreparationDenial {
    pub const fn kind(&self) -> WorthQueryWorkflowDefinitionRetirementPreparationDenialKind {
        match self {
            Self::RuntimeMismatch => {
                WorthQueryWorkflowDefinitionRetirementPreparationDenialKind::RuntimeMismatch
            }
            Self::RequestAdmission(_) => {
                WorthQueryWorkflowDefinitionRetirementPreparationDenialKind::RequestAdmission
            }
            Self::DefinitionPreparation(_) => {
                WorthQueryWorkflowDefinitionRetirementPreparationDenialKind::DefinitionPreparation
            }
        }
    }
}

impl std::fmt::Display for WorthQueryWorkflowDefinitionRetirementPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "workflow definition retirement preparation denied: {:?}",
            self.kind()
        )
    }
}

impl std::error::Error for WorthQueryWorkflowDefinitionRetirementPreparationDenial {}

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
    /// Retires one exact current definition so its lineage admits no new
    /// starts. History, custody, and instances pinned to it are untouched.
    pub fn prepare_workflow_definition_retirement<'workflow, Spec, Program: 'workflow>(
        mut self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec, Program>>,
        definition: PublishedWorkflowDefinitionRef,
    ) -> Result<
        WorthQueryWorkflowDefinitionRetirementRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        PreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let workflow = workflow.into();
        let application = self.application_runtime();
        if !std::ptr::eq(application, workflow.runtime()) {
            return Err(PreparationDenial::RuntimeMismatch);
        }
        let selected = application
            .on_branch(self.product_branch())
            .select()
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)
            .map_err(PreparationDenial::RequestAdmission)?;
        let mutation = authorization::prepare_capability_selected(&mut self, &selected)
            .map_err(PreparationDenial::RequestAdmission)?;
        let prepared = WorthQueryWorkflowDefinitionRetirementAdapter::prepare::<
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
            mutation.admission,
        )
        .map_err(PreparationDenial::DefinitionPreparation)?;
        Ok(WorthQueryWorkflowDefinitionRetirementRequest {
            application,
            prepared,
            idempotency: mutation.idempotency,
        })
    }
}

pub struct WorthQueryWorkflowDefinitionRetirementRequest<
    'application,
    Schema,
    Operation,
    Input,
    Scope,
> where
    Schema: ApplicationSchema,
{
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    prepared: PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>,
    idempotency: WorthQueryApplicationIdempotencyBinding,
}

impl<Schema, Operation, Input, Scope>
    WorthQueryWorkflowDefinitionRetirementRequest<'_, Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn execute(self) -> WorkflowDefinitionRetirementOutcome {
        WorthQueryWorkflowDefinitionRetirementAdapter::compare_and_commit(
            self.application,
            self.prepared,
            self.idempotency,
        )
    }
}
