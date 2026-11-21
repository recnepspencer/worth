//! Original typed command locators. This carries neither an actor nor a grant.
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::actor_context::*;
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::ApplicationWorkflowSpec,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};
use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::actor_context::{
    Command, OperationActorCommand,
};

impl WorthQueryWorkflowAdvanceAdapter {
    /// Own only original locators from the exact installed vocabulary; authorization stays fresh.
    pub fn bind_owned_operation_actor_context<Schema, Spec, Binding>(
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        workflow: crate::domain_computation::primary_graph::application_installation::WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Option<WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        if required.original_runtime() != workflow.runtime().runtime.authority_identity().as_u64() {
            return Err(affinity("foreign workflow actor runtime"));
        }
        let installed = workflow
            .workflow_spec_at_revision(required.original_program_revision())
            .ok_or_else(|| {
                affinity("original workflow actor vocabulary revision is unavailable")
            })?;
        Self::bind_operation_actor_context::<Schema, Spec, Binding>(access, installed, required)
    }

    /// Publication alone supplies the original typed command before admission.
    /// The context is an untrusted locator; native selection checks its facts.
    pub fn bind_approval_actor_context<Schema, Spec, Binding>(
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        workflow: crate::domain_computation::primary_graph::application_installation::WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        instance: &super::super::PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &super::super::PublishedWorkflowProposalRef,
    ) -> Result<Option<WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let command = Command::Approval {
            required: required.clone(),
            proposal: proposal.clone(),
        };
        WorthQueryWorkflowActorContext::build::<Schema, Spec, Binding>(
            worth_query_execution_attempt::facade::ports::WorthQueryAttemptCrateSeal::new(),
            selected.application().runtime.authority_identity().as_u64(),
            workflow.workflow_spec_for(selected),
            instance,
            command,
        )
    }

    pub fn bind_operation_actor_context<Schema, Spec, Binding>(
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Option<WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        WorthQueryWorkflowActorContext::build::<Schema, Spec, Binding>(
            worth_query_execution_attempt::facade::ports::WorthQueryAttemptCrateSeal::new(),
            required.original_runtime(),
            installed,
            required.original_instance(),
            Command::Operation {
                required: OperationActorCommand {
                    captured_input: required.captured_input.clone(),
                    instance: required.instance(),
                    branch: required.branch(),
                    node_path: required.node_path().to_owned(),
                    operation: required.operation().to_owned(),
                    binding: required.binding().map(str::to_owned),
                    input_identity: *required.input_identity(),
                    original_sources: required.original_sources().cloned(),
                },
            },
        )
    }
}

fn affinity(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
