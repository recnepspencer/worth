use bank_domain::{
    proposals::{BankIdempotencyKey, BankProposalDenial},
    schema::{
        ApprovePayment, ApprovePaymentMutationBinding, ApprovedBusinessPaymentApplyIntent,
        ApprovedBusinessPaymentApprovalIntent, ApprovedBusinessPaymentAuthoringIntent,
        ApprovedBusinessPaymentInstanceStartIntent, BankSchema,
    },
};
use worth_query_host::facade::primary_graph::WorthQueryExternalDispatchPostureKind;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef,
        RequiredWorkflowApproval, RequiredWorkflowOperation, WorkflowApprovalDecision,
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
        WorthQueryApplicationMutationOutcome, WorthQueryPreparedWorkflowOperationRecovery,
    },
    primary_graph::{WorthQueryApplicationCommitReceipt, WorthQueryApplicationUncommitted},
};

use crate::{
    approved_business_payment_definition, BankApprovalCredential, BankAuthenticatedPrincipal,
    BankIdentityRuntime,
};

#[path = "approved_payment_workflow/assessment.rs"]
mod assessment;
#[path = "approved_payment_workflow/condition.rs"]
mod condition;
#[path = "approved_payment_workflow/error.rs"]
mod error;
mod native_operation;
#[path = "approved_payment_workflow/owner.rs"]
mod owner;
mod owning;
pub use native_operation::BankAdvancedApprovedPaymentWorkflow;
pub use owning::BankCompletedApprovedPayment;
#[path = "approved_payment_workflow/progression.rs"]
mod progression;
pub use assessment::{
    BankApprovedPaymentAssessment, BankApprovedPaymentAssessmentDemand,
    BankApprovedPaymentAssessmentProgress, BankPaymentAssessmentSettlement,
};
pub use error::BankApprovedPaymentWorkflowError;
pub use progression::BankApprovedPaymentCancellation;

pub type BankApprovedPaymentPreparedRecovery<'runtime> =
    WorthQueryPreparedWorkflowOperationRecovery<
        'runtime,
        BankSchema,
        ApprovePaymentMutationBinding,
    >;

#[derive(Debug)]
pub struct BankApprovedPaymentPerformedOperation {
    receipt: WorthQueryApplicationCommitReceipt,
    newly_committed: bool,
}

#[derive(Debug)]
pub enum BankApprovedPaymentApplyOutcome {
    Performed(BankApprovedPaymentPerformedOperation),
    PreviouslyCommitted(
        worth_query_host::facade::primary_graph::WorthQueryHistoricalApplicationCommit,
    ),
    IdempotencyIntentDrift,
    DomainDenied(BankProposalDenial),
    Cancelled,
    DeadlineExceeded,
    Commit(WorthQueryApplicationUncommitted),
}

impl BankApprovedPaymentApplyOutcome {
    pub fn into_performed(
        self,
    ) -> Result<BankApprovedPaymentPerformedOperation, Box<BankApprovedPaymentApplyOutcome>> {
        match self {
            Self::Performed(performed) => Ok(performed),
            other => Err(Box::new(other)),
        }
    }
}

impl BankApprovedPaymentPerformedOperation {
    pub const fn newly_committed(&self) -> bool {
        self.newly_committed
    }

    pub const fn emitted_effect_count(&self) -> usize {
        self.receipt.emitted_effect_count()
    }

    pub fn co_committed_dispatch_outbox(&self) -> bool {
        self.receipt.dispatch_outbox().is_some()
    }

    pub fn external_dispatch_posture(&self) -> Option<WorthQueryExternalDispatchPostureKind> {
        self.receipt
            .external_dispatch()
            .map(|dispatch| dispatch.posture().kind())
    }
}

pub struct BankApprovedPaymentWorkflow<'runtime, 'principal, 'scope> {
    runtime: &'runtime BankIdentityRuntime,
    principal: &'principal BankAuthenticatedPrincipal,
    scope: &'scope WorthQueryRequestScope,
}

impl<'runtime, 'principal, 'scope> BankApprovedPaymentWorkflow<'runtime, 'principal, 'scope> {
    pub(crate) const fn new(
        runtime: &'runtime BankIdentityRuntime,
        principal: &'principal BankAuthenticatedPrincipal,
        scope: &'scope WorthQueryRequestScope,
    ) -> Self {
        Self {
            runtime,
            principal,
            scope,
        }
    }

    pub fn publish_definition(
        &self,
        authority: ApprovePayment,
        expected_predecessor: WorkflowDefinitionExpectedPredecessor,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowDefinitionPublicationOutcome, BankApprovedPaymentWorkflowError> {
        let contract = self
            .runtime
            .approved_payment_workflow_runtime()
            .workflow_spec()
            .bind_definition(
                approved_business_payment_definition()
                    .map_err(BankApprovedPaymentWorkflowError::Definition)?,
            )
            .map_err(BankApprovedPaymentWorkflowError::DefinitionBinding)?;
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAuthoringIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_publication(contract, expected_predecessor)
            .map(|request| request.execute())
            .map_err(BankApprovedPaymentWorkflowError::DefinitionPublication)
    }

    pub fn start(
        &self,
        definition: PublishedWorkflowDefinitionRef,
        authority: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowInstanceStartOutcome, BankApprovedPaymentWorkflowError> {
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentInstanceStartIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_instance_start(
                self.runtime.approved_payment_workflow_runtime(),
                definition,
            )
            .map(|request| request.execute())
            .map_err(BankApprovedPaymentWorkflowError::InstanceStart)
    }

    pub fn propose(
        &self,
        instance: PublishedWorkflowInstanceRef,
        proposal: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowProposalOutcome, BankApprovedPaymentWorkflowError> {
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAuthoringIntent { input: proposal })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_proposal(self.runtime.approved_payment_workflow_runtime(), instance)
            .map(|request| request.execute())
            .map_err(BankApprovedPaymentWorkflowError::Proposal)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn approve(
        &self,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
        credential: BankApprovalCredential,
        authority: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowProgressOutcome, BankApprovedPaymentWorkflowError> {
        let signing = self
            .runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentApprovalIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_approval(
                self.runtime.approved_payment_workflow_runtime(),
                instance,
                required,
                proposal,
                decision,
            )
            .map_err(BankApprovedPaymentWorkflowError::Advance)?;
        let Some(intent) = signing.authentication_intent().cloned() else {
            return signing
                .execute_replay()
                .map_err(BankApprovedPaymentWorkflowError::Advance);
        };
        let event = self
            .runtime
            .approval_authentication()
            .authenticate(credential, self.principal.external(), intent, self.scope)
            .await
            .map_err(BankApprovedPaymentWorkflowError::Authentication)?;
        signing
            .sign(&event)
            .map(|request| request.execute())
            .map_err(BankApprovedPaymentWorkflowError::Advance)
    }

    /// Retains the exact guarded effect without retaining this principal or request scope.
    pub fn retain_apply(
        &self,
        required: &RequiredWorkflowOperation,
        operation: ApprovePayment,
        command_key: BankIdempotencyKey,
    ) -> Result<
        worth_query_host::facade::application_entry::WorthQueryWorkflowOperation<
            BankSchema,
            ApprovedBusinessPaymentApplyIntent,
            crate::BankApplication,
            crate::ApprovedPaymentEffectAction,
        >,
        BankApprovedPaymentWorkflowError,
    > {
        self.runtime
            .request(self.principal, self.scope)
            .on_branch(required.branch())
            .mutate(ApprovedBusinessPaymentApplyIntent { input: operation })
            .without_source()
            .idempotency_owned(command_key)
            .for_workflow_operation(self.runtime.approved_payment_workflow_runtime(), required)
            .map_err(|(_, cause)| BankApprovedPaymentWorkflowError::OperationBinding(cause))?
            .retain_operation::<crate::BankApplication, crate::ApprovedPaymentEffectAction>(
                self.runtime.application_program(),
            )
            .map_err(|(_, cause)| BankApprovedPaymentWorkflowError::OperationConstruction(cause))
    }
    /// Current native workflow controls for callers explicitly managing the native funnel.
    pub fn advanced(&self) -> BankAdvancedApprovedPaymentWorkflow<'runtime, 'principal, 'scope> {
        BankAdvancedApprovedPaymentWorkflow {
            runtime: self.runtime,
            principal: self.principal,
            scope: self.scope,
        }
    }
}
