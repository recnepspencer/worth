use crate::{
    estate::EstateCaseId,
    model::{AccountId, BankPrincipalId},
    proposals::BankIdempotencyKey,
    schema::*,
};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_structured_value_binding,
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

pub struct RestoreFrozenEstateAccountBinding;
impl ApplicationMutationBinding<BankSchema> for RestoreFrozenEstateAccountBinding {
    type Input = RestoreFrozenEstateAccount;
    type InputBinding = RestoreFrozenEstateAccountInputBinding;
    type Result = EstateMutationResult;
    type ResultBinding = EstateMutationResultBinding;
    type IdempotencyKey = BankIdempotencyKey;
    type Operation = RestoreFrozenEstateAccountOperation;
    type Decision = RestoreFrozenEstateAccount;
    type Denial = EstateMutationDenial;
    type DenialBinding = EstateMutationDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = <FreezeEstateAccountMutationBinding as ApplicationMutationBinding<
        BankSchema,
    >>::ScopeBinding;
    type PrincipalBinding = BankPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = BankPrincipalId;
    type PrincipalIdentityBinding = BankPrincipalIdBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "bank.estate.restore-frozen-account.binding.v1";
    const HANDLER_IDENTITY: &'static str = "bank.estate.restore-frozen-account.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "bank.application-mutation-client-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;

    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        BankSchema,
        EstateCase,
        EstateCaseRecord,
        EstateCaseIdentityField,
        EstateCaseId,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        EstateCaseIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        BankSchema,
        BankPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        BankPrincipalId,
        BankPrincipalIdBinding,
    > {
        BankPrincipalBinding::reference()
    }
}
impl ApplicationCapabilityMutationBinding<BankSchema> for RestoreFrozenEstateAccountBinding {
    type Capability = RestoreFrozenEstateAccountCapability;
}
impl ApplicationMutationAdmissionBinding<BankSchema> for RestoreFrozenEstateAccountBinding {
    type Admission = ApplicationCapabilityAdmission<RestoreFrozenEstateAccountCapability>;
}
impl ApplicationMutationIntent<BankSchema> for RestoreFrozenEstateAccount {
    type Binding = RestoreFrozenEstateAccountBinding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<BankSchema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(EstateCaseIdentityField::reference(), self.estate)
    }
}
