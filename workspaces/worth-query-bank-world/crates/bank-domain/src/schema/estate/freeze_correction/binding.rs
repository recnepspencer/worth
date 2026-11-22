use crate::{estate::EstateCaseId, model::AccountId, schema::*};
use worth_query_decl::facade::{
    application_operation::{
        ApplicationCapabilityAdmission, ApplicationCapabilityMutationBinding,
        NoApplicationMutationOutputs,
    },
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct RestoreFrozenEstateAccount {
    pub estate: EstateCaseId,
    pub account: AccountId,
    pub prior_status: AccountStatus,
}
worth_query_structured_value_binding!(
    pub RestoreFrozenEstateAccountInputBinding for RestoreFrozenEstateAccount {
        identity: "bank.estate.restore-frozen-account.input.v1"
    }
);

fn restore_scope(input: &RestoreFrozenEstateAccount) -> EstateCaseId {
    input.estate
}

worth_query_mutation_binding!(
    pub RestoreFrozenEstateAccountBinding for RestoreFrozenEstateAccount, schema BankSchema,
    context BankBindingContext,
    identity "bank.estate.restore-frozen-account.binding.v1",
    input RestoreFrozenEstateAccountInputBinding,
    operation RestoreFrozenEstateAccountOperation,
    result EstateMutationResultBinding,
    decision RestoreFrozenEstateAccount,
    denial EstateMutationDenialBinding,
    handler identity "bank.estate.restore-frozen-account.handler.v1",
    admission ApplicationCapabilityAdmission<RestoreFrozenEstateAccountCapability>,
    program required,
    outputs NoApplicationMutationOutputs,
    scope EstateCaseIdentityField,
    field EstateCaseIdentityField::at("EstateCaseIdentityField"),
    value restore_scope,
);

impl ApplicationCapabilityMutationBinding<BankSchema> for RestoreFrozenEstateAccountBinding {
    type Capability = RestoreFrozenEstateAccountCapability;
}
