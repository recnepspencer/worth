use bank_domain::model::PaymentId;
use bank_domain::reads::PaymentSummary;
use bank_domain::schema::PaymentStatus;

/// A descriptive pending-payment handle. It carries no workflow authority.
///
/// Callers may reconstruct it from a payment identity received across a
/// process boundary. Rejection still requires a fresh authenticated principal,
/// request scope, installed admission, projection, and invariant decision;
/// approval runs only through the approved-payment workflow.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankPendingPaymentContinuation {
    payment: PaymentId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankPaymentContinuationDenial {
    PaymentNotApprovalRequired,
}

/// Rejecting actor identity is derived only after a fresh principal is supplied.
///
/// ```compile_fail
/// use bank_domain::model::PaymentId;
/// use bank_server::BankRejectPendingPayment;
///
/// let payment = PaymentId::new(1).unwrap();
/// let _ = BankRejectPendingPayment { payment };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankRejectPendingPayment {
    pub(in crate::ordinary::mutation) payment: PaymentId,
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

    pub const fn reject(self) -> BankRejectPendingPayment {
        BankRejectPendingPayment {
            payment: self.payment,
        }
    }
}
