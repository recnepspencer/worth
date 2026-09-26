//! Bank settlement of one approved-payment assessment demand.
//!
//! Settling never blocks. Each round spends at most the policy's attempts,
//! and another round runs only when the demand was notified during the last
//! one; a quiet demand reports `Pending` so the caller can settle it again.

use std::num::NonZeroUsize;

use bank_domain::{
    proposals::BankIdempotencyKey,
    queries::PaymentDetailQuery,
    schema::{
        ApprovePayment, ApprovedBusinessPaymentAdvanceIntent, ApprovedBusinessPaymentWorkflow,
        ApprovedPaymentAssessmentDemand, BankSchema,
    },
};
use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, RequiredWorkflowAssessment, WorkflowProgressOutcome,
    WorthQueryOutputDemandControls, WorthQueryWorkflowAssessmentDemandHandle,
    WorthQueryWorkflowAssessmentDemandProgress, WorthQueryWorkflowAssessmentDemandSettlement,
};

use super::{BankApprovedPaymentWorkflow, BankApprovedPaymentWorkflowError};
use crate::application_definition::BankApplication;

pub type BankApprovedPaymentAssessment =
    WorthQueryWorkflowAssessmentDemandSettlement<PaymentDetailQuery>;
pub type BankApprovedPaymentAssessmentProgress =
    WorthQueryWorkflowAssessmentDemandProgress<PaymentDetailQuery>;

const ASSESSMENT_WORK: NonZeroUsize = NonZeroUsize::new(1_024).expect("work is nonzero");
const ASSESSMENT_BYTES: NonZeroUsize = NonZeroUsize::new(16_384).expect("bytes are nonzero");

/// How much one settle call may spend: `attempts` source readings per round
/// and at most `rounds` rounds, each after the first only when notified.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankPaymentAssessmentSettlement {
    attempts: NonZeroUsize,
    rounds: NonZeroUsize,
}

impl BankPaymentAssessmentSettlement {
    pub const DEFAULT: Self = Self::new(
        NonZeroUsize::new(64).expect("attempts are nonzero"),
        NonZeroUsize::new(4).expect("rounds are nonzero"),
    );

    pub const fn new(attempts: NonZeroUsize, rounds: NonZeroUsize) -> Self {
        Self { attempts, rounds }
    }

    pub const fn attempts(self) -> NonZeroUsize {
        self.attempts
    }

    pub const fn rounds(self) -> NonZeroUsize {
        self.rounds
    }
}

/// One started payment assessment and the settlement policy it keeps.
pub struct BankApprovedPaymentAssessmentDemand<'runtime> {
    handle: WorthQueryWorkflowAssessmentDemandHandle<
        'runtime,
        BankSchema,
        ApprovedBusinessPaymentWorkflow,
        BankApplication,
        ApprovedPaymentAssessmentDemand,
    >,
    settlement: BankPaymentAssessmentSettlement,
}

impl BankApprovedPaymentAssessmentDemand<'_> {
    pub const fn required(&self) -> &RequiredWorkflowAssessment {
        self.handle.required()
    }

    pub const fn settlement(&self) -> BankPaymentAssessmentSettlement {
        self.settlement
    }

    pub fn close(&mut self) {
        self.handle.close();
    }
}

impl<'runtime> BankApprovedPaymentWorkflow<'runtime, '_, '_> {
    pub fn begin_payment_assessment(
        &self,
        instance: PublishedWorkflowInstanceRef,
        authority: ApprovePayment,
        settlement: BankPaymentAssessmentSettlement,
        command_key: &BankIdempotencyKey,
    ) -> Result<BankApprovedPaymentAssessmentDemand<'runtime>, BankApprovedPaymentWorkflowError>
    {
        let payment_id = authority.payment;
        let handle = self
            .runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance)
            .map_err(BankApprovedPaymentWorkflowError::Advance)?
            .into_assessment_demand(ApprovedPaymentAssessmentDemand::new(payment_id))
            .map_err(BankApprovedPaymentWorkflowError::AssessmentPreparation)?
            .controls(
                WorthQueryOutputDemandControls::new(ASSESSMENT_WORK, ASSESSMENT_BYTES)
                    .settlement_attempts(settlement.attempts),
            )
            .start()
            .map_err(BankApprovedPaymentWorkflowError::AssessmentDemand)?;
        Ok(BankApprovedPaymentAssessmentDemand { handle, settlement })
    }

    pub fn settle_payment_assessment(
        &self,
        demand: &mut BankApprovedPaymentAssessmentDemand<'runtime>,
    ) -> Result<BankApprovedPaymentAssessmentProgress, BankApprovedPaymentWorkflowError> {
        let notifications = demand
            .handle
            .notifications()
            .map_err(BankApprovedPaymentWorkflowError::AssessmentDemand)?;
        let request = self.runtime.request(self.principal, self.scope);
        for _ in 0..demand.settlement.rounds.get() {
            let observed = notifications.generation();
            let progress = demand
                .handle
                .settle(&request)
                .map_err(BankApprovedPaymentWorkflowError::AssessmentDemand)?;
            if matches!(
                progress,
                WorthQueryWorkflowAssessmentDemandProgress::Settled(_)
            ) || notifications.generation() == observed
            {
                return Ok(progress);
            }
        }
        Ok(WorthQueryWorkflowAssessmentDemandProgress::Pending)
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
}
