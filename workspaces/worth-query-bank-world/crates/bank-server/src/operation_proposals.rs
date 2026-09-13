mod account_access;
mod account_creation;
mod business_payment;
mod money_movement;
mod reversal;

use bank_domain::proposals::{BankInvariantApprovedProposal, BankProposalDenial};
use bank_domain::schema::BankSchema;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationOperationInvariantProjectionSnapshot, WorthQueryInvariantProjectionWork,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationProjectionDenial,
    WorthQueryOperationProjectionDenialKind,
};

use crate::bank_projection::BankProjectionDenial;
use crate::BankAdmittedOperation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankOperationProposalError {
    Authorization(crate::BankAuthorizationDenial),
    AuthorizationLineageUnavailable(crate::BankAuthorizationDenial),
    InvariantAdmission(
        worth_query_host::facade::primary_graph::WorthQueryInvariantProjectionDenialKind,
    ),
    ProjectionWorkBudgetExceeded,
    Projection(BankProjectionDenial),
    Invariant(BankProposalDenial),
    Idempotency(crate::BankIdempotencyResolutionDenialKind),
}

impl From<WorthQueryOperationAuthorizationDenial> for BankOperationProposalError {
    fn from(denial: WorthQueryOperationAuthorizationDenial) -> Self {
        Self::Authorization(crate::BankAuthorizationDenial::from_query(denial))
    }
}

impl From<WorthQueryOperationProjectionDenial> for BankOperationProposalError {
    fn from(denial: WorthQueryOperationProjectionDenial) -> Self {
        let kind = denial.kind();
        match kind {
            WorthQueryOperationProjectionDenialKind::Authorization(authorization_kind) => denial
                .into_authorization_denial()
                .map(|denial| {
                    Self::Authorization(crate::BankAuthorizationDenial::from_query(denial))
                })
                .unwrap_or(Self::AuthorizationLineageUnavailable(
                    crate::BankAuthorizationDenial::from_kind(authorization_kind, 0),
                )),
            WorthQueryOperationProjectionDenialKind::WorkBudgetExceeded => {
                Self::ProjectionWorkBudgetExceeded
            }
            WorthQueryOperationProjectionDenialKind::InvariantAdmission(kind) => {
                Self::InvariantAdmission(kind)
            }
        }
    }
}

impl From<BankProjectionDenial> for BankOperationProposalError {
    fn from(denial: BankProjectionDenial) -> Self {
        Self::Projection(denial)
    }
}

impl From<BankProposalDenial> for BankOperationProposalError {
    fn from(denial: BankProposalDenial) -> Self {
        Self::Invariant(denial)
    }
}

impl std::fmt::Display for BankOperationProposalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authorization(kind) => {
                write!(
                    formatter,
                    "bank projection authorization denied: {:?}",
                    kind.kind()
                )
            }
            Self::AuthorizationLineageUnavailable(kind) => {
                write!(
                    formatter,
                    "bank projection authorization lineage unavailable: {:?}",
                    kind.kind()
                )
            }
            Self::ProjectionWorkBudgetExceeded => {
                formatter.write_str("bank projection work budget exceeded")
            }
            Self::InvariantAdmission(kind) => {
                write!(
                    formatter,
                    "bank invariant projection admission denied: {kind:?}"
                )
            }
            Self::Projection(denial) => denial.fmt(formatter),
            Self::Invariant(denial) => denial.fmt(formatter),
            Self::Idempotency(kind) => {
                write!(formatter, "bank idempotency resolution denied: {kind:?}")
            }
        }
    }
}

impl std::error::Error for BankOperationProposalError {}

/// Typed phase progression retaining both installed operation admission and
/// the bank-domain invariant witness for the future compare-and-commit phase.
///
/// ```compile_fail
/// use bank_server::BankAuthorizedProposal;
///
/// let _ = BankAuthorizedProposal::<(), (), (), u64> {
///     admission: todo!(),
///     invariant: todo!(),
/// };
/// ```
pub struct BankAuthorizedProposal<Operation, Input, Scope, ScopeIdentity> {
    admission: BankAdmittedOperation<Operation, Input, Scope, ScopeIdentity>,
    invariant: BankInvariantApprovedProposal,
    projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<BankSchema, Operation>,
    projection_work: WorthQueryInvariantProjectionWork,
}

impl<Operation, Input, Scope, ScopeIdentity>
    BankAuthorizedProposal<Operation, Input, Scope, ScopeIdentity>
{
    pub(crate) const fn new_bounded(
        admission: BankAdmittedOperation<Operation, Input, Scope, ScopeIdentity>,
        invariant: BankInvariantApprovedProposal,
        projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<
            BankSchema,
            Operation,
        >,
        projection_work: WorthQueryInvariantProjectionWork,
    ) -> Self {
        Self {
            admission,
            invariant,
            projection,
            projection_work,
        }
    }

    pub const fn admission(
        &self,
    ) -> &BankAdmittedOperation<Operation, Input, Scope, ScopeIdentity> {
        &self.admission
    }

    pub const fn invariant(&self) -> &BankInvariantApprovedProposal {
        &self.invariant
    }

    pub const fn projection_work(&self) -> crate::BankMutationProjectionWork {
        crate::BankMutationProjectionWork::from_query(self.projection_work)
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        BankAdmittedOperation<Operation, Input, Scope, ScopeIdentity>,
        BankInvariantApprovedProposal,
        WorthQueryApplicationOperationInvariantProjectionSnapshot<BankSchema, Operation>,
    ) {
        (self.admission, self.invariant, self.projection)
    }
}

/// Installed operation projection is a compile-time capability, not a
/// schema-wide reader.
///
/// `PaymentAmount` is intentionally available to payment operations but not
/// to `SendMoneyOperation`, so this attempted projection cannot compile:
///
/// ```compile_fail,E0277
/// use bank_domain::schema::{
///     BankSchema, PaymentAmount, PaymentIntent, SendMoneyOperation,
/// };
/// use worth_query_host::facade::primary_graph::{
///     WorthQueryApplicationOperationInvariantProjectionReader,
///     WorthQueryInvariantEntityIdentity,
/// };
///
/// fn read_undeclared_payment_amount(
///     reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
///         '_, '_, BankSchema, SendMoneyOperation,
///     >,
///     payment: &WorthQueryInvariantEntityIdentity<BankSchema, PaymentIntent>,
/// ) {
///     let _ = reader.field(payment, PaymentAmount::reference());
/// }
/// ```
pub struct BankOperationProposals;

pub enum BankSendMoneyPreparation {
    Proposal(
        BankAuthorizedProposal<
            bank_domain::schema::SendMoneyOperation,
            bank_domain::schema::SendMoney,
            bank_domain::schema::Account,
            bank_domain::model::AccountId,
        >,
    ),
    AlreadyCommitted {
        receipt: crate::BankCommitReceipt,
        projection_work: crate::BankMutationProjectionWork,
    },
    IntentDrift {
        projection_work: crate::BankMutationProjectionWork,
    },
}
