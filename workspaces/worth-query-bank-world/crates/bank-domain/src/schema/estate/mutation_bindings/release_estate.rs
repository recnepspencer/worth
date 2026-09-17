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
    estate::{EstateAction, EstateCaseId, LegalAuthorityId, MandatoryReviewId},
    model::BankPrincipalId,
    proposals::{BankIdempotencyKey, CanonicalProposalPayload},
    schema::{
        BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, EstateCase,
        EstateCaseIdentityField, ExternalPrincipalMapping, Principal, ReleaseEstateCapability,
        ReleaseEstateOperation,
    },
};

use super::{
    EstateMutationDenial, EstateMutationDenialBinding, EstateMutationResult,
    EstateMutationResultBinding,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReleaseEstate {
    action: EstateAction,
    estate: EstateCaseId,
}

impl ReleaseEstate {
    pub const fn new(
        estate: EstateCaseId,
        executor: BankPrincipalId,
        authority: LegalAuthorityId,
        review: MandatoryReviewId,
    ) -> Self {
        Self {
            action: EstateAction::ReleaseEstate {
                estate,
                executor,
                authority,
                review,
            },
            estate,
        }
    }
}

pub struct ReleaseEstateMutationBinding;

impl ApplicationMutationBinding<BankSchema> for ReleaseEstateMutationBinding {
    type Input = EstateAction;
    type InputBinding = super::super::EstateActionInputBinding;
    type Result = EstateMutationResult;
    type ResultBinding = EstateMutationResultBinding;
    type IdempotencyKey = BankIdempotencyKey;
    type Operation = ReleaseEstateOperation;
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

    const IDENTITY: &'static str = "bank.estate.release.mutation-binding.v1";
    const HANDLER_IDENTITY: &'static str = "bank.estate.release.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "bank.application-mutation-client-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 1, 0),
            ApplicationCandidateResourceCeiling::bounded(32768, 33),
        );

    fn idempotency_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
        crate::schema::operations::client_key_identity(key)
    }

    fn input_identity(input: &EstateAction) -> [u8; 32] {
        let EstateAction::ReleaseEstate {
            estate,
            executor,
            authority,
            review,
        } = input
        else {
            return super::invalid_variant_identity("application-release-estate", input);
        };
        *CanonicalProposalPayload::new("application-release-estate")
            .u64("estate", estate.get())
            .u64("executor", executor.get())
            .u64("authority", authority.get())
            .u64("review", review.get())
            .derive_identity()
            .bytes()
    }

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

impl ApplicationCapabilityMutationBinding<BankSchema> for ReleaseEstateMutationBinding {
    type Capability = ReleaseEstateCapability;
}

impl ApplicationMutationIntent<BankSchema> for ReleaseEstate {
    type Binding = ReleaseEstateMutationBinding;

    fn input(&self) -> &EstateAction {
        &self.action
    }

    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<BankSchema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(EstateCaseIdentityField::reference(), self.estate)
    }
}
