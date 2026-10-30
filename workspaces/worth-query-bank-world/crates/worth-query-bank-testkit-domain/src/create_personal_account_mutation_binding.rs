pub use std::any::TypeId;

pub use bank_domain::{
    model::{AccountId, AccountName, BankPrincipalId, InstitutionId},
    proposals::{BankIdempotencyKey, BankProposalDenial},
    schema::{
        BankSchema, CreatePersonalAccount, CreatePersonalAccountDecision,
        CreatePersonalAccountMutationBinding, CreatePersonalAccountOperation,
        CreatePersonalAccountResult, CreatePersonalAccountResultBinding,
    },
};
pub use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationIntent,
        ApplicationMutationScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};

pub fn personal_account_input(
    institution: u64,
    owner: u64,
    display_name: &str,
) -> CreatePersonalAccount {
    CreatePersonalAccount {
        institution: InstitutionId::new(institution).unwrap(),
        owner: BankPrincipalId::new(owner).unwrap(),
        display_name: AccountName::new(display_name).unwrap(),
    }
}
