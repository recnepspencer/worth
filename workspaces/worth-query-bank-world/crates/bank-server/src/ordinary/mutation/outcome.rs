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
pub type BankMoneyMovementExecution =
    BankProgramMutationExecution<bank_domain::schema::MoneyMovementResult>;
