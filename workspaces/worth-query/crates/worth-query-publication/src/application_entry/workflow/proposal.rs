use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary;
use worth_query_execution::publication_boundary::workflow_proposal::{
    PreparedWorkflowProposal, PublishedWorkflowInstanceRef, WorkflowProposalOutcome,
    WorkflowProposalPreparationDenial, WorthQueryWorkflowProposalAdapter,
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

/// The kind of a `WorthQueryWorkflowProposalPreparationDenial`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowProposalPreparationDenialKind {
    RuntimeMismatch,
    RequestAdmission,
    ProposalPreparation,
    IdempotencyResolution,
    InstanceBranchMismatch,
    Inputs,
}

/// Why `prepare_workflow_proposal` refused. `InstanceBranchMismatch` means the instance is
/// on another branch than the request.
#[derive(Debug)]
pub enum WorthQueryWorkflowProposalPreparationDenial {
    RuntimeMismatch,
    RequestAdmission(WorthQueryApplicationRequestMutationDenial),
    ProposalPreparation(WorkflowProposalPreparationDenial),
    IdempotencyResolution(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
    InstanceBranchMismatch,
    Inputs(crate::application_entry::WorthQueryWorkflowInputDenial),
}

impl WorthQueryWorkflowProposalPreparationDenial {
    pub const fn kind(&self) -> WorthQueryWorkflowProposalPreparationDenialKind {
        match self {
            Self::Inputs(_) => WorthQueryWorkflowProposalPreparationDenialKind::Inputs,
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
    pub fn prepare_workflow_proposal<'workflow, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
    ) -> Result<WorthQueryWorkflowProposalRequest<'application, Schema, MutationOperation<Schema, Intent>, MutationInput<Schema, Intent>, MutationScope<Schema, IntentBinding<Schema, Intent>>>, WorthQueryWorkflowProposalPreparationDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.prepare_workflow_proposal_inputs(workflow, instance, |_, _, _| Ok(Vec::new()))
    }

    /// Publish the complete original target-input contracts with this proposal.
    /// Capturing is descriptive: target readiness and authorization are checked
    /// only by their later ordinary guarded-operation admissions.
    pub fn prepare_workflow_proposal_with_inputs<'workflow, Spec, Program, R>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        inputs: &crate::application_entry::WorthQueryWorkflowOperationInputs<Schema, Program, R>,
    ) -> Result<WorthQueryWorkflowProposalRequest<'application, Schema, MutationOperation<Schema, Intent>, MutationInput<Schema, Intent>, MutationScope<Schema, IntentBinding<Schema, Intent>>>, WorthQueryWorkflowProposalPreparationDenial>
    where Schema: 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Program: worth_query_declaration::facade::application_program::ApplicationProgramDefinition<Schema>,
        R: crate::application_entry::WorthQueryWorkflowInputList<Schema, Program>,
    {
        self.prepare_workflow_proposal_inputs(workflow, instance, |request, phase, selected| {
            inputs.prepare_capture(request, phase, selected)
        })
    }

    fn prepare_workflow_proposal_inputs<'workflow, Spec>(
        mut self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        capture: impl FnOnce(
            &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
            &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
            &worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<'_, Schema>,
        ) -> Result<Vec<(worth_query_declaration::facade::application_program::ApplicationWorkflowNodeIdentity,
            worth_query_execution::publication_boundary::workflow_proposal::PreparedWorkflowCapturedInput)>, crate::application_entry::WorthQueryWorkflowInputDenial>,
    ) -> Result<WorthQueryWorkflowProposalRequest<'application, Schema, MutationOperation<Schema, Intent>, MutationInput<Schema, Intent>, MutationScope<Schema, IntentBinding<Schema, Intent>>>, WorthQueryWorkflowProposalPreparationDenial>
    where Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let request_scope = self.request_scope().clone();
        let runtime = self.application_runtime();
        runtime.with_application_advancement(&request_scope, |phase| {

        let workflow = workflow.into();
        let application = self.application_runtime();
        let request_branch = self.product_branch();
        if !std::ptr::eq(application, workflow.runtime()) {
            return Err(WorthQueryWorkflowProposalPreparationDenial::RuntimeMismatch);
        }
        let selected = application.on_branch(self.product_branch()).select(&phase, )
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)
            .map_err(WorthQueryWorkflowProposalPreparationDenial::RequestAdmission)?;
        let staged = self.stage().map_err(WorthQueryWorkflowProposalPreparationDenial::RequestAdmission)?;
        let identities = self
            .identities()
            .map_err(WorthQueryWorkflowProposalPreparationDenial::RequestAdmission)?;
        let mut mutation = authorization::prepare_capability_selected(&phase, &self, &identities, staged, &selected)
            .map_err(WorthQueryWorkflowProposalPreparationDenial::RequestAdmission)?;
        let fresh = crate::application_entry::WorthQueryApplicationRequest {
            application, principal: self.authenticated_principal(), scope: self.request_scope(), branch: request_branch,
        };
        let captured = capture(&fresh, &phase, &selected)
            .map_err(WorthQueryWorkflowProposalPreparationDenial::Inputs)?;
        let access = worth_query_execution::publication_boundary::program_publication_access();
        let captured = worth_query_execution::publication_boundary::workflow_proposal::WorkflowCapturedInputs::prepare::<Schema, IntentBinding<Schema, Intent>, _>(
            &access, &selected, &mut mutation.admission, captured,
        ).map_err(|cause| WorthQueryWorkflowProposalPreparationDenial::Inputs(
            crate::application_entry::WorthQueryWorkflowInputDenial::Capture(cause)))?;
        mutation.idempotency = captured.bind_idempotency(&access, mutation.idempotency);
        if instance.branch() != request_branch {
            return Err(WorthQueryWorkflowProposalPreparationDenial::InstanceBranchMismatch);
        }
        if let Some(outcome) = WorthQueryWorkflowProposalAdapter::resolve_replay_with_inputs(&phase,
            application,
            &mutation.admission,
            mutation.idempotency,
            instance.clone(),
            &captured,
        )
        .map_err(WorthQueryWorkflowProposalPreparationDenial::IdempotencyResolution)?
        {
            return Ok(WorthQueryWorkflowProposalRequest {
                execution: WorkflowProposalRequestExecution::Resolved(outcome),
            });
        }
        let prepared = WorthQueryWorkflowProposalAdapter::prepare_with_inputs(&phase,
            &selected,
            workflow.workflow_spec_for(&selected),
            instance,
            mutation.admission,
            &mutation.idempotency,
            captured,
        ).map_err(WorthQueryWorkflowProposalPreparationDenial::ProposalPreparation)?;
        Ok(WorthQueryWorkflowProposalRequest {
            execution: WorkflowProposalRequestExecution::Prepared {
                application,
                prepared: Box::new(prepared),
                idempotency: mutation.idempotency,
            },
        })

        }).map_err(|cause| WorthQueryWorkflowProposalPreparationDenial::RequestAdmission(
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
        ))?
    }
}

/// A prepared workflow proposal. `execute` attempts its commit.
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
        prepared: Box<PreparedWorkflowProposal<Schema, Operation, Input, Scope>>,
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
            } => application
                .with_application_advancement(&prepared.request_scope().clone(), |phase| {
                    WorthQueryWorkflowProposalAdapter::compare_and_commit(
                        &phase,
                        application,
                        *prepared,
                        idempotency,
                    )
                })
                .unwrap_or_else(|cause| {
                    WorkflowProposalOutcome::Application(
                        cause
                            .into_commit_outcome()
                            .landed()
                            .expect_err("request admission cannot commit"),
                    )
                }),
            WorkflowProposalRequestExecution::Resolved(outcome) => outcome,
        }
    }
}

mod ordinary;
