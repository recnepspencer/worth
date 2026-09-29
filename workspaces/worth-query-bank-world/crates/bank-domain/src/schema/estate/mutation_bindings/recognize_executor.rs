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
};

use crate::{
    estate::{EstateAction, EstateCaseId, LegalAuthorityId},
    model::BankPrincipalId,
    proposals::BankIdempotencyKey,
    schema::{
        BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, EstateCase,
        EstateCaseIdentityField, ExternalPrincipalMapping, Principal,
        RecognizeEstateExecutorCapability, RecognizeEstateExecutorOperation,
    },
};

use super::{
    EstateMutationDenial, EstateMutationDenialBinding, EstateMutationResult,
    EstateMutationResultBinding,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecognizeEstateExecutor {
    action: EstateAction,
    estate: EstateCaseId,
}

impl RecognizeEstateExecutor {
    pub const fn new(
        estate: EstateCaseId,
        executor: BankPrincipalId,
        authority: LegalAuthorityId,
    ) -> Self {
        Self {
            action: EstateAction::RecognizeExecutor {
                estate,
                executor,
                authority,
            },
            estate,
        }
    }
}

pub struct RecognizeEstateExecutorMutationBinding;

impl ApplicationMutationBinding<BankSchema> for RecognizeEstateExecutorMutationBinding {
    type Input = EstateAction;
    type InputBinding = super::super::EstateActionInputBinding;
    type Result = EstateMutationResult;
    type ResultBinding = EstateMutationResultBinding;
    type IdempotencyKey = BankIdempotencyKey;
    type Operation = RecognizeEstateExecutorOperation;
    type Decision = EstateAction;
    type Denial = EstateMutationDenial;
    type DenialBinding = EstateMutationDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<
        BankSchema,
        EstateCase,
        super::super::EstateCaseRecord,
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

    const IDENTITY: &'static str = "bank.estate.recognize-executor.mutation-binding.v1";
    const HANDLER_IDENTITY: &'static str = "bank.estate.recognize-executor.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "bank.application-mutation-client-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 1, 0, 0, 0),
            ApplicationCandidateResourceCeiling::bounded(32768, 7),
        );

    fn scope_field() -> ApplicationFieldRef<
        BankSchema,
        EstateCase,
        super::super::EstateCaseRecord,
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

impl ApplicationCapabilityMutationBinding<BankSchema> for RecognizeEstateExecutorMutationBinding {
    type Capability = RecognizeEstateExecutorCapability;
}

impl ApplicationMutationIntent<BankSchema> for RecognizeEstateExecutor {
    type Binding = RecognizeEstateExecutorMutationBinding;

    fn input(&self) -> &EstateAction {
        &self.action
    }

    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<BankSchema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(EstateCaseIdentityField::reference(), self.estate)
    }
}
