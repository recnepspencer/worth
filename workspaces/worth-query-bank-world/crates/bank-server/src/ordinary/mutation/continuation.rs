use bank_domain::model::PaymentId;
use bank_domain::reads::PaymentSummary;
use bank_domain::schema::PaymentStatus;

use super::BankProgramMutationExecution;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

/// A descriptive pending-payment handle. It carries no workflow authority.
///
/// Callers may reconstruct it from a payment identity received across a
/// process boundary. Approval or rejection still requires a fresh
/// authenticated principal, request scope, installed admission, projection,
/// and invariant decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankPendingPaymentContinuation {
    payment: PaymentId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankPaymentContinuationDenial {
    PaymentNotApprovalRequired,
}

/// Approval actor identity is derived only after a fresh principal is supplied.
///
/// ```compile_fail
/// use bank_domain::model::PaymentId;
/// use bank_server::BankApprovePendingPayment;
///
/// let payment = PaymentId::new(1).unwrap();
/// let _ = BankApprovePendingPayment { payment };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankApprovePendingPayment {
    pub(in crate::ordinary::mutation) payment: PaymentId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankRejectPendingPayment {
    pub(in crate::ordinary::mutation) payment: PaymentId,
}

#[derive(Debug)]
pub struct BankPaymentInitiationOutcome {
    execution: BankProgramMutationExecution<bank_domain::schema::InitiateBusinessPaymentResult>,
    continuation: Option<BankPendingPaymentContinuation>,
}

impl BankPendingPaymentContinuation {
    pub fn from_summary(summary: PaymentSummary) -> Result<Self, BankPaymentContinuationDenial> {
        if summary.status() != PaymentStatus::ApprovalRequired {
            return Err(BankPaymentContinuationDenial::PaymentNotApprovalRequired);
        }
        Ok(Self::from_payment_id(summary.id()))
    }

    pub const fn from_payment_id(payment: PaymentId) -> Self {
        Self { payment }
    }

    pub const fn payment_id(self) -> PaymentId {
        self.payment
    }

    pub const fn approve(self) -> BankApprovePendingPayment {
        BankApprovePendingPayment {
            payment: self.payment,
        }
    }

    pub const fn reject(self) -> BankRejectPendingPayment {
        BankRejectPendingPayment {
            payment: self.payment,
        }
    }
}

impl BankPaymentInitiationOutcome {
    pub(super) fn from_program(
        execution: BankProgramMutationExecution<bank_domain::schema::InitiateBusinessPaymentResult>,
    ) -> Self {
        let continuation = match &execution {
            Ok(WorthQueryApplicationMutationOutcome::Committed { result, .. }) => Some(
                BankPendingPaymentContinuation::from_payment_id(result.payment),
            ),
            _ => None,
        };
        Self {
            execution,
            continuation,
        }
    }

    pub const fn execution(
        &self,
    ) -> &BankProgramMutationExecution<bank_domain::schema::InitiateBusinessPaymentResult> {
        &self.execution
    }

    pub const fn continuation(&self) -> Option<BankPendingPaymentContinuation> {
        self.continuation
    }

    pub fn into_continuation(self) -> Option<BankPendingPaymentContinuation> {
        self.continuation
    }

    pub fn into_execution(
        self,
    ) -> BankProgramMutationExecution<bank_domain::schema::InitiateBusinessPaymentResult> {
        self.execution
    }
}
