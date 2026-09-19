mod admission_denial;

pub use admission_denial::{
    BankAuthorizationDenial, BankAuthorizationDenialKind, BankEntityResolutionDenial,
    BankEntityResolutionDenialKind, BankOperationInstallationDenial,
    BankOperationInstallationDenialKind,
};

use bank_domain::proposals::BankProposalDenial;

pub type BankProgramMutationExecution<Output> = Result<
    worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome<
        BankProposalDenial,
        Output,
    >,
    worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial,
>;
pub type BankPaymentDecisionExecution =
    BankProgramMutationExecution<bank_domain::schema::PaymentDecisionResult>;
pub type BankPersonalAccountCreationExecution =
    BankProgramMutationExecution<bank_domain::schema::CreatePersonalAccountResult>;
pub type BankBusinessAccountCreationExecution =
    BankProgramMutationExecution<bank_domain::schema::CreateBusinessAccountResult>;
pub type BankAccountAccessExecution =
    BankProgramMutationExecution<bank_domain::schema::AccountAccessResult>;
pub type BankMoneyMovementExecution =
    BankProgramMutationExecution<bank_domain::schema::MoneyMovementResult>;
pub type BankMoneyMovementRetainedExecution = Result<
    worth_query_host::facade::application_entry::WorthQueryApplicationRetainedMutationOutcome<
        BankProposalDenial,
        bank_domain::schema::MoneyMovementResult,
    >,
    worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial,
>;
