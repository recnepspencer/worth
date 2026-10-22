//! Bank settlement of one approved-payment assessment demand.
//!
//! Settling never blocks. Each round spends at most the policy's attempts,
//! and another round runs only when the demand was notified during the last
//! one; a quiet demand reports `Pending` so the caller can settle it again.
//! A completed stage returns `Progressed` immediately. The caller may renew its
//! actor authority before advancing the same retained demand again.

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
    WorthQueryOutputDemandControls, WorthQueryWorkflowAssessmentDemandProgress,
    WorthQueryWorkflowAssessmentDemandSettlement, WorthQueryWorkflowAssessmentOperation,
};

use super::{BankApprovedPaymentWorkflow, BankApprovedPaymentWorkflowError};

pub type BankApprovedPaymentAssessment =
    WorthQueryWorkflowAssessmentDemandSettlement<PaymentDetailQuery>;
pub type BankApprovedPaymentAssessmentProgress =
    WorthQueryWorkflowAssessmentDemandProgress<PaymentDetailQuery>;

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

/// One retained payment assessment and the settlement policy it keeps.
/// Native observer driving is not an ordinary assessment entrance.
/// ```compile_fail,E0599
/// fn cannot_start(request:worth_query_host::facade::application_entry::WorthQueryWorkflowAssessmentDemandRequest<'_,'_,'_,bank_domain::schema::BankSchema,bank_domain::schema::ApprovedBusinessPaymentWorkflow,bank_domain::schema::ApprovedPaymentAssessmentDemand>) { request.start(); }
/// ```
/// The actual installed Bank program retains the admitted demand instead.
/// ```no_run
/// fn retain(request:worth_query_host::facade::application_entry::WorthQueryWorkflowAssessmentDemandRequest<'_,'_,'_,bank_domain::schema::BankSchema,bank_domain::schema::ApprovedBusinessPaymentWorkflow,bank_domain::schema::ApprovedPaymentAssessmentDemand>,runtime:&worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<bank_domain::schema::BankSchema,bank_server::BankApplication>) { let _=request.retain_operation(runtime); }
/// ```
pub struct BankApprovedPaymentAssessmentDemand {
    handle: WorthQueryWorkflowAssessmentOperation<
        BankSchema,
        ApprovedBusinessPaymentWorkflow,
        ApprovedPaymentAssessmentDemand,
        crate::BankApplication,
    >,
    settlement: BankPaymentAssessmentSettlement,
}

impl BankApprovedPaymentAssessmentDemand {
    pub fn required(&self) -> &RequiredWorkflowAssessment {
        self.handle.required()
    }

    pub const fn settlement(&self) -> BankPaymentAssessmentSettlement {
        self.settlement
    }

    pub fn close(
        self,
        request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
            '_,
            '_,
            '_,
            BankSchema,
        >,
    ) -> Result<
        worth_query_host::facade::application_entry::WorthQueryClosedAssessmentObservation<
            BankSchema,
            ApprovedPaymentAssessmentDemand,
            crate::BankApplication,
        >,
        (
            Self,
            worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial,
        ),
    > {
        let Self { handle, settlement } = self;
        handle
            .close(request)
            .map_err(|(handle, cause)| (Self { handle, settlement }, cause))
    }
}

impl<'runtime> BankApprovedPaymentWorkflow<'runtime, '_, '_> {
    pub fn retain_payment_assessment(
        &self,
        instance: PublishedWorkflowInstanceRef,
        authority: ApprovePayment,
        settlement: BankPaymentAssessmentSettlement,
        command_key: &BankIdempotencyKey,
    ) -> Result<BankApprovedPaymentAssessmentDemand, BankApprovedPaymentWorkflowError> {
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
                WorthQueryOutputDemandControls::default().settlement_attempts(settlement.attempts),
            )
            .retain_operation(self.runtime.application_program())
            .map_err(|denial| {
                BankApprovedPaymentWorkflowError::AssessmentDemand(Box::new(denial))
            })?;
        Ok(BankApprovedPaymentAssessmentDemand { handle, settlement })
    }

    pub fn advance_payment_assessment(
        &self,
        demand: &mut BankApprovedPaymentAssessmentDemand,
    ) -> Result<BankApprovedPaymentAssessmentProgress, BankApprovedPaymentWorkflowError> {
        let notifications = demand.handle.notifications().map_err(|denial| {
            BankApprovedPaymentWorkflowError::AssessmentDemand(Box::new(denial))
        })?;
        let request = self
            .runtime
            .request(self.principal, self.scope)
            .on_branch(demand.handle.branch());
        for _ in 0..demand.settlement.rounds.get() {
            let observed = notifications.generation();
            let progress = demand.handle.advance(&request).map_err(|denial| {
                BankApprovedPaymentWorkflowError::AssessmentDemand(Box::new(denial))
            })?;
            if matches!(
                progress,
                WorthQueryWorkflowAssessmentDemandProgress::Settled(_)
                    | WorthQueryWorkflowAssessmentDemandProgress::Progressed
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
