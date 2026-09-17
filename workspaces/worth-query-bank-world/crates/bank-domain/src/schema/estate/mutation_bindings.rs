use worth_query_decl::facade::{
    application_operation::{
        ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
        ApplicationCandidateResourceCeiling, ApplicationCapabilityMutationBinding,
        ApplicationMutationBinding, ApplicationMutationFieldScope, ApplicationMutationIntent,
        NoApplicationMutationOutputs, NoApplicationMutationSource,
    },
    application_schema::{
        ApplicationFieldRef, ApplicationPrincipalBindingRef, EqualityPredicate, NoApplicationUnit,
        ReadOnly,
    },
    worth_query_structured_value_binding,
};

use crate::{
    estate::{DeathNoticeId, EstateAction, EstateCaseId},
    model::{AccountId, BankPrincipalId},
    proposals::{BankIdempotencyKey, CanonicalProposalPayload},
};

use crate::schema::{
    BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, EstateCase, EstateCaseIdentityField,
    ExternalPrincipalMapping, FreezeEstateAccountCapability, FreezeEstateAccountOperation,
    NotifyDeathEstateCapability, NotifyDeathEstateOperation, Principal,
};

mod open_estate_case;
pub use open_estate_case::*;
mod recognize_executor;
pub use recognize_executor::*;
mod release_estate;
pub use release_estate::*;
mod action_identity;
mod disburse_estate;
mod workflow_idempotency;
pub use disburse_estate::*;
mod retransmit_death_notice;
pub use retransmit_death_notice::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NotifyEstateDeath {
    action: EstateAction,
    estate: EstateCaseId,
}

impl NotifyEstateDeath {
    pub const fn new(
        estate: EstateCaseId,
        notice: DeathNoticeId,
        subject: BankPrincipalId,
    ) -> Self {
        Self {
            action: EstateAction::NotifyDeath {
                estate,
                notice,
                subject,
            },
            estate,
        }
    }

    pub const fn action(self) -> EstateAction {
        self.action
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FreezeEstateAccount {
    action: EstateAction,
    estate: EstateCaseId,
}

impl FreezeEstateAccount {
    pub const fn new(estate: EstateCaseId, account: AccountId) -> Self {
        Self {
            action: EstateAction::FreezeAccount { estate, account },
            estate,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateMutationResult {
    pub estate: EstateCaseId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EstateMutationDenial {
    InputVariant,
    Proposal(crate::proposals::BankProposalDenial),
}

worth_query_structured_value_binding!(
    pub EstateMutationResultBinding for EstateMutationResult {
        identity: "bank.estate.mutation-result.v1"
    }
);
worth_query_structured_value_binding!(
    pub EstateMutationDenialBinding for EstateMutationDenial {
        identity: "bank.estate.mutation-denial.v1"
    }
);

pub struct NotifyEstateDeathMutationBinding;

impl ApplicationMutationBinding<BankSchema> for NotifyEstateDeathMutationBinding {
    type Input = EstateAction;
    type InputBinding = super::EstateActionInputBinding;
    type Result = EstateMutationResult;
    type ResultBinding = EstateMutationResultBinding;
    type IdempotencyKey = BankIdempotencyKey;
    type Operation = NotifyDeathEstateOperation;
    type Decision = EstateAction;
    type Denial = EstateMutationDenial;
    type DenialBinding = EstateMutationDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<
        BankSchema,
        EstateCase,
        super::EstateCaseRecord,
        EstateCaseIdentityField,
        EstateCaseId,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = BankPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = BankPrincipalId;
    type PrincipalIdentityBinding = BankPrincipalIdBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "bank.estate.notify-death.mutation-binding.v1";
    const HANDLER_IDENTITY: &'static str = "bank.estate.notify-death.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "bank.application-mutation-client-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 1, 1),
            ApplicationCandidateResourceCeiling::bounded(32768, 8),
        );

    fn idempotency_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
        super::super::operations::client_key_identity(key)
    }

    fn input_identity(input: &EstateAction) -> [u8; 32] {
        let EstateAction::NotifyDeath {
            estate,
            notice,
            subject,
        } = input
        else {
            return invalid_variant_identity("application-notify-estate-death", input);
        };
        *CanonicalProposalPayload::new("application-notify-estate-death")
            .u64("estate", estate.get())
            .u64("notice", notice.get())
            .u64("subject", subject.get())
            .derive_identity()
            .bytes()
    }

    fn scope_field() -> ApplicationFieldRef<
        BankSchema,
        EstateCase,
        super::EstateCaseRecord,
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

pub struct FreezeEstateAccountMutationBinding;

impl ApplicationMutationBinding<BankSchema> for FreezeEstateAccountMutationBinding {
    type Input = EstateAction;
    type InputBinding = super::EstateActionInputBinding;
    type Result = EstateMutationResult;
    type ResultBinding = EstateMutationResultBinding;
    type IdempotencyKey = BankIdempotencyKey;
    type Operation = FreezeEstateAccountOperation;
    type Decision = EstateAction;
    type Denial = EstateMutationDenial;
    type DenialBinding = EstateMutationDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<
        BankSchema,
        EstateCase,
        super::EstateCaseRecord,
        EstateCaseIdentityField,
        EstateCaseId,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = BankPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = BankPrincipalId;
    type PrincipalIdentityBinding = BankPrincipalIdBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "bank.estate.freeze-account.mutation-binding.v1";
    const HANDLER_IDENTITY: &'static str = "bank.estate.freeze-account.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "bank.application-mutation-client-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 1, 0),
            ApplicationCandidateResourceCeiling::bounded(32768, 4),
        );

    fn idempotency_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
        super::super::operations::client_key_identity(key)
    }

    fn input_identity(input: &EstateAction) -> [u8; 32] {
        let EstateAction::FreezeAccount { estate, account } = input else {
            return invalid_variant_identity("application-freeze-estate-account", input);
        };
        *CanonicalProposalPayload::new("application-freeze-estate-account")
            .u64("estate", estate.get())
            .text("account", &account.canonical_text())
            .derive_identity()
            .bytes()
    }

    fn scope_field() -> ApplicationFieldRef<
        BankSchema,
        EstateCase,
        super::EstateCaseRecord,
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

impl ApplicationCapabilityMutationBinding<BankSchema> for FreezeEstateAccountMutationBinding {
    type Capability = FreezeEstateAccountCapability;
}

impl ApplicationMutationIntent<BankSchema> for FreezeEstateAccount {
    type Binding = FreezeEstateAccountMutationBinding;

    fn input(&self) -> &EstateAction {
        &self.action
    }

    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<BankSchema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(EstateCaseIdentityField::reference(), self.estate)
    }
}

pub(super) fn invalid_variant_identity(operation: &'static str, input: &EstateAction) -> [u8; 32] {
    *action_identity::canonical_action_payload(operation, input)
        .text("input-variant", "mismatch")
        .derive_identity()
        .bytes()
}

impl ApplicationCapabilityMutationBinding<BankSchema> for NotifyEstateDeathMutationBinding {
    type Capability = NotifyDeathEstateCapability;
}

impl ApplicationMutationIntent<BankSchema> for NotifyEstateDeath {
    type Binding = NotifyEstateDeathMutationBinding;

    fn input(&self) -> &EstateAction {
        &self.action
    }

    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<BankSchema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(EstateCaseIdentityField::reference(), self.estate)
    }
}
