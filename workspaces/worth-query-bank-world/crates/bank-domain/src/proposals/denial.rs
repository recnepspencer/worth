use crate::model::{AccountId, JournalEntryId, PaymentId};
use crate::schema::AccountStatus;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankProposalDenial {
    UnknownInstitution,
    UnknownPrincipal,
    UnknownBusiness,
    UnknownAccount(AccountId),
    UnknownRecipient,
    UnknownPayment(PaymentId),
    UnknownJournal(JournalEntryId),
    UnknownAuthorization,
    MissingInstitutionCashAccount,
    DuplicatePersonalAccount,
    DuplicateBusinessAccount,
    DuplicateAuthorization,
    AccountAlreadyFunded(AccountId),
    AccountStatus {
        account: AccountId,
        status: AccountStatus,
    },
    AccountInstitutionMismatch,
    AccountOwnershipMismatch,
    InsufficientFunds(AccountId),
    SelfTransfer,
    SelfApproval,
    TooManyPaymentApprovers,
    PaymentAlreadyDecided(PaymentId),
    JournalAlreadyReversed(JournalEntryId),
    JournalHasTooFewPostings,
    JournalIsUnbalanced,
    DisbursementPostingMismatch,
    ArithmeticOverflow,
    IdentityExhausted,
    InvalidIdempotencyKey,
    ScopeInputMismatch,
    AuthenticatedActorMismatch,
    SnapshotInvariantViolated,
}

impl std::fmt::Display for BankProposalDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for BankProposalDenial {}
