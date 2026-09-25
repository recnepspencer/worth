use bank_domain::{
    proposals::{BankIdempotencyKey, BankProposalDenial},
    queries::PaymentDetailQuery,
    schema::{
        ApprovePayment, ApprovePaymentMutationBinding, ApprovedBusinessPaymentAdvanceIntent,
        ApprovedBusinessPaymentApplyIntent, ApprovedBusinessPaymentApprovalIntent,
        ApprovedBusinessPaymentAuthoringIntent, ApprovedBusinessPaymentInstanceStartIntent,
        ApprovedPaymentAssessmentDemand, BankSchema,
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
        WorthQueryApplicationMutationOutcome, WorthQueryOutputDemandControls,
        WorthQueryPreparedWorkflowOperationRecovery, WorthQueryWorkflowAssessmentDemandHandle,
        WorthQueryWorkflowAssessmentDemandProgress, WorthQueryWorkflowAssessmentDemandSettlement,
    },
    primary_graph::{WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt},
};

use crate::{
    approved_business_payment_definition, BankApprovalCredential, BankAuthenticatedPrincipal,
    BankIdentityRuntime,
};

#[path = "approved_payment_workflow/error.rs"]
mod error;
#[path = "approved_payment_workflow/owner.rs"]
mod owner;
#[path = "approved_payment_workflow/progression.rs"]
mod progression;
pub use error::BankApprovedPaymentWorkflowError;

pub type BankApprovedPaymentAssessment =
    WorthQueryWorkflowAssessmentDemandSettlement<PaymentDetailQuery>;
pub type BankApprovedPaymentAssessmentDemand<'runtime> = WorthQueryWorkflowAssessmentDemandHandle<
    'runtime,
    BankSchema,
    bank_domain::schema::ApprovedBusinessPaymentWorkflow,
    crate::application_definition::BankApplication,
    ApprovedPaymentAssessmentDemand,
>;
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
    IdempotencyIntentDrift,
    DomainDenied(BankProposalDenial),
    Cancelled,
    DeadlineExceeded,
    Commit(WorthQueryApplicationCommitOutcome),
}

impl BankApprovedPaymentApplyOutcome {
    pub fn into_performed(
        self,
    ) -> Result<BankApprovedPaymentPerformedOperation, BankApprovedPaymentApplyOutcome> {
        match self {
            Self::Performed(performed) => Ok(performed),
            other => Err(other),
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

    pub fn begin_payment_assessment(
        &self,
        instance: PublishedWorkflowInstanceRef,
        authority: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<BankApprovedPaymentAssessmentDemand<'runtime>, BankApprovedPaymentWorkflowError>
    {
        let payment_id = authority.payment;
        let request = self.runtime.request(self.principal, self.scope);
        request
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance)
            .map_err(BankApprovedPaymentWorkflowError::Advance)?
            .into_assessment_demand(ApprovedPaymentAssessmentDemand::new(payment_id))
            .map_err(BankApprovedPaymentWorkflowError::AssessmentPreparation)?
            .controls(WorthQueryOutputDemandControls::new(
                std::num::NonZeroUsize::new(1_024).expect("assessment work is nonzero"),
                std::num::NonZeroUsize::new(16_384).expect("assessment bytes are nonzero"),
            ))
            .start()
            .map_err(BankApprovedPaymentWorkflowError::AssessmentDemand)
    }

    pub fn settle_payment_assessment(
        &self,
        demand: &mut BankApprovedPaymentAssessmentDemand<'runtime>,
    ) -> Result<
        WorthQueryWorkflowAssessmentDemandProgress<PaymentDetailQuery>,
        BankApprovedPaymentWorkflowError,
    > {
        demand
            .settle(&self.runtime.request(self.principal, self.scope))
            .map_err(BankApprovedPaymentWorkflowError::AssessmentDemand)
    }

    pub fn accept_assessment(
        &self,
        instance: PublishedWorkflowInstanceRef,
        authority: ApprovePayment,
        assessment: &BankApprovedPaymentAssessment,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowProgressOutcome, BankApprovedPaymentWorkflowError> {
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance)
            .map_err(BankApprovedPaymentWorkflowError::Advance)?
            .accept_assessment(assessment)
            .map_err(BankApprovedPaymentWorkflowError::AssessmentAcceptance)
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

    pub fn perform_apply(
        &self,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowOperation,
        operation: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<BankApprovedPaymentApplyOutcome, BankApprovedPaymentWorkflowError> {
        let effect = self
            .runtime
            .request(self.principal, self.scope)
            .on_branch(instance.branch())
            .mutate(ApprovedBusinessPaymentApplyIntent {
                input: operation.clone(),
            })
            .idempotency(command_key)
            .for_workflow_operation(self.runtime.approved_payment_workflow_runtime(), required)
            .map_err(BankApprovedPaymentWorkflowError::OperationBinding)?
            .execute_in_program(self.runtime.approved_payment_workflow_runtime())
            .map_err(BankApprovedPaymentWorkflowError::OperationMutation)?;
        Ok(match effect {
            WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => {
                BankApprovedPaymentApplyOutcome::Performed(BankApprovedPaymentPerformedOperation {
                    receipt,
                    newly_committed: true,
                })
            }
            WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt) => {
                BankApprovedPaymentApplyOutcome::Performed(BankApprovedPaymentPerformedOperation {
                    receipt,
                    newly_committed: false,
                })
            }
            WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift => {
                BankApprovedPaymentApplyOutcome::IdempotencyIntentDrift
            }
            WorthQueryApplicationMutationOutcome::DomainDenied(denial) => {
                BankApprovedPaymentApplyOutcome::DomainDenied(denial)
            }
            WorthQueryApplicationMutationOutcome::Cancelled => {
                BankApprovedPaymentApplyOutcome::Cancelled
            }
            WorthQueryApplicationMutationOutcome::DeadlineExceeded => {
                BankApprovedPaymentApplyOutcome::DeadlineExceeded
            }
            WorthQueryApplicationMutationOutcome::Commit(outcome) => {
                BankApprovedPaymentApplyOutcome::Commit(outcome)
            }
        })
    }
}
