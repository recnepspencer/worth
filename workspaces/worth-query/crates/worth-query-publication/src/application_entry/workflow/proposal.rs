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
    application_installation::WorthQueryWorkflowApplicationRuntime,
    workflow_proposal::{
        PreparedWorkflowProposal, PublishedWorkflowInstanceRef, WorkflowProposalOutcome,
        WorkflowProposalPreparationDenial, WorthQueryWorkflowProposalAdapter,
    },
};
use worth_query_installation::facade::ApplicationSchema;

use crate::application_entry::{
    mutation::{authorization, WorthQueryApplicationMutationRequestWithIdempotency},
    WorthQueryApplicationRequestMutationDenial,
};

type IntentBinding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope;
type MutationOperation<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Operation;
type MutationInput<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Input;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowProposalPreparationDenialKind {
    RuntimeMismatch,
    RequestAdmission,
    ProposalPreparation,
    IdempotencyResolution,
    InstanceBranchMismatch,
}

#[derive(Debug)]
pub enum WorthQueryWorkflowProposalPreparationDenial {
    RuntimeMismatch,
    RequestAdmission(WorthQueryApplicationRequestMutationDenial),
    ProposalPreparation(WorkflowProposalPreparationDenial),
    IdempotencyResolution(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
    InstanceBranchMismatch,
}

impl WorthQueryWorkflowProposalPreparationDenial {
    pub const fn kind(&self) -> WorthQueryWorkflowProposalPreparationDenialKind {
        match self {
            Self::RuntimeMismatch => {
                WorthQueryWorkflowProposalPreparationDenialKind::RuntimeMismatch
            }
            Self::RequestAdmission(_) => {
                WorthQueryWorkflowProposalPreparationDenialKind::RequestAdmission
            }
            Self::ProposalPreparation(_) => {
                WorthQueryWorkflowProposalPreparationDenialKind::ProposalPreparation
            }
            Self::IdempotencyResolution(_) => {
                WorthQueryWorkflowProposalPreparationDenialKind::IdempotencyResolution
            }
            Self::InstanceBranchMismatch => {
                WorthQueryWorkflowProposalPreparationDenialKind::InstanceBranchMismatch
            }
        }
    }
}

impl std::fmt::Display for WorthQueryWorkflowProposalPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "workflow proposal preparation denied: {:?}",
            self.kind()
        )
    }
}

impl std::error::Error for WorthQueryWorkflowProposalPreparationDenial {}

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<Schema, <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
    MutationInput<Schema, Intent>: Clone + Send + Sync + 'static
        + ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
{
    pub fn prepare_workflow_proposal<Spec, Program>(
        mut self,
        workflow: &WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>,
        instance: PublishedWorkflowInstanceRef,
    ) -> Result<WorthQueryWorkflowProposalRequest<'application, Schema, MutationOperation<Schema, Intent>, MutationInput<Schema, Intent>, MutationScope<Schema, IntentBinding<Schema, Intent>>>, WorthQueryWorkflowProposalPreparationDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let application = self.application_runtime();
        let request_branch = self.product_branch();
        if !std::ptr::eq(application, workflow.program_runtime().runtime()) {
            return Err(WorthQueryWorkflowProposalPreparationDenial::RuntimeMismatch);
        }
        let selected = application.on_branch(self.product_branch()).select()
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)
            .map_err(WorthQueryWorkflowProposalPreparationDenial::RequestAdmission)?;
        let mutation = authorization::prepare_capability_selected(&mut self, &selected)
            .map_err(WorthQueryWorkflowProposalPreparationDenial::RequestAdmission)?;
        if instance.branch() != request_branch {
            return Err(WorthQueryWorkflowProposalPreparationDenial::InstanceBranchMismatch);
        }
        if let Some(outcome) = WorthQueryWorkflowProposalAdapter::resolve_replay(
            application,
            &mutation.admission,
            mutation.idempotency,
            instance.clone(),
        )
        .map_err(WorthQueryWorkflowProposalPreparationDenial::IdempotencyResolution)?
        {
            return Ok(WorthQueryWorkflowProposalRequest {
                execution: WorkflowProposalRequestExecution::Resolved(outcome),
            });
        }
        let prepared = WorthQueryWorkflowProposalAdapter::prepare(
            &selected,
            workflow.workflow_spec(),
            instance,
            mutation.admission,
            &mutation.idempotency,
        ).map_err(WorthQueryWorkflowProposalPreparationDenial::ProposalPreparation)?;
        Ok(WorthQueryWorkflowProposalRequest {
            execution: WorkflowProposalRequestExecution::Prepared {
                application,
                prepared,
                idempotency: mutation.idempotency,
            },
        })
    }
}

pub struct WorthQueryWorkflowProposalRequest<'application, Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
{
    execution: WorkflowProposalRequestExecution<'application, Schema, Operation, Input, Scope>,
}

enum WorkflowProposalRequestExecution<'application, Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
{
    Prepared {
        application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowProposal<Schema, Operation, Input, Scope>,
        idempotency: worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding,
    },
    Resolved(WorkflowProposalOutcome),
}

impl<Schema, Operation, Input, Scope>
    WorthQueryWorkflowProposalRequest<'_, Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn execute(self) -> WorkflowProposalOutcome {
        match self.execution {
            WorkflowProposalRequestExecution::Prepared {
                application,
                prepared,
                idempotency,
            } => WorthQueryWorkflowProposalAdapter::compare_and_commit(
                application,
                prepared,
                idempotency,
            ),
            WorkflowProposalRequestExecution::Resolved(outcome) => outcome,
        }
    }
}
