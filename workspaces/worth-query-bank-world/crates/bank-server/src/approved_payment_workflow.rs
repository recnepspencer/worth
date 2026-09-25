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
        WorthQueryPreparedWorkflowOperationRecovery, WorthQueryWorkflowAssessmentDemandProgress,
        WorthQueryWorkflowAssessmentDemandSettlement,
    },
    primary_graph::{WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt},
};

use crate::{
    approved_business_payment_definition, BankAuthenticatedPrincipal, BankIdentityRuntime,
};

pub type BankApprovedPaymentAssessment =
    WorthQueryWorkflowAssessmentDemandSettlement<PaymentDetailQuery>;
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
            .bind_definition(approved_business_payment_definition().map_err(debug_error)?)
            .map_err(debug_error)?;
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAuthoringIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_publication(contract, expected_predecessor)
            .map(|request| request.execute())
            .map_err(debug_error)
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
            .map_err(debug_error)
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
            .map_err(debug_error)
    }

    pub fn advance(
        &self,
        instance: PublishedWorkflowInstanceRef,
        authority: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowProgressOutcome, BankApprovedPaymentWorkflowError> {
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance)
            .map(|request| request.execute())
            .map_err(debug_error)
    }

    pub fn settle_payment_assessment(
        &self,
        instance: PublishedWorkflowInstanceRef,
        authority: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<BankApprovedPaymentAssessment, BankApprovedPaymentWorkflowError> {
        let payment_id = authority.payment;
        let request = self.runtime.request(self.principal, self.scope);
        let mut demand = request
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance)
            .map_err(debug_error)?
            .into_assessment_demand(ApprovedPaymentAssessmentDemand::new(payment_id))
            .map_err(debug_error)?
            .controls(WorthQueryOutputDemandControls::new(
                std::num::NonZeroUsize::new(1_024).expect("assessment work is nonzero"),
                std::num::NonZeroUsize::new(16_384).expect("assessment bytes are nonzero"),
            ))
            .start()
            .map_err(debug_error)?;
        for _ in 0..8 {
            match demand.settle(&request).map_err(debug_error)? {
                WorthQueryWorkflowAssessmentDemandProgress::Pending => {}
                WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => {
                    return Ok(settled);
                }
            }
        }
        Err(BankApprovedPaymentWorkflowError(
            "payment assessment exceeded its settlement work".to_owned(),
        ))
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
            .map_err(debug_error)?
            .accept_assessment(assessment)
            .map_err(debug_error)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn approve(
        &self,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
        authority: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowProgressOutcome, BankApprovedPaymentWorkflowError> {
        self.runtime
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
            .map(|request| request.execute())
            .map_err(debug_error)
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
            .map_err(debug_error)?
            .execute_in_program(self.runtime.approved_payment_workflow_runtime())
            .map_err(debug_error)?;
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

    pub fn accept_applied(
        &self,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowOperation,
        authority: ApprovePayment,
        performed: &BankApprovedPaymentPerformedOperation,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowProgressOutcome, BankApprovedPaymentWorkflowError> {
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance)
            .map_err(debug_error)?
            .accept_operation::<ApprovePaymentMutationBinding>(required, &performed.receipt)
            .map_err(debug_error)
    }

    pub fn prepare_apply_recovery(
        &self,
        required: &RequiredWorkflowOperation,
        operation: ApprovePayment,
        performed: &BankApprovedPaymentPerformedOperation,
        command_key: &BankIdempotencyKey,
    ) -> Result<BankApprovedPaymentPreparedRecovery<'runtime>, BankApprovedPaymentWorkflowError>
    {
        self.runtime
            .request(self.principal, self.scope)
            .on_branch(performed.receipt.product_branch())
            .mutate(ApprovedBusinessPaymentApplyIntent { input: operation })
            .idempotency(command_key)
            .for_workflow_operation(self.runtime.approved_payment_workflow_runtime(), required)
            .map_err(debug_error)?
            .prepare_workflow_operation_recovery(&performed.receipt)
            .map_err(debug_error)
    }

    pub fn accept_recovered_applied(
        &self,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowOperation,
        authority: ApprovePayment,
        performed: &BankApprovedPaymentPerformedOperation,
        recovery: &worth_query_host::facade::primary_graph::WorthQueryRecoverySafeRetryAdmission,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowProgressOutcome, BankApprovedPaymentWorkflowError> {
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance)
            .map_err(debug_error)?
            .accept_recovered_operation::<ApprovePaymentMutationBinding>(
                required,
                &performed.receipt,
                recovery,
            )
            .map_err(debug_error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankApprovedPaymentWorkflowError(String);

impl std::fmt::Display for BankApprovedPaymentWorkflowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for BankApprovedPaymentWorkflowError {}

fn debug_error(error: impl std::fmt::Debug) -> BankApprovedPaymentWorkflowError {
    BankApprovedPaymentWorkflowError(format!("{error:?}"))
}
