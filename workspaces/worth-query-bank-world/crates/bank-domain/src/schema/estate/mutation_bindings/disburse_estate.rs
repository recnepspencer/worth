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
    estate::{EstateAction, EstateCaseId, EstateDisbursement},
    model::BankPrincipalId,
    proposals::{BankIdempotencyKey, BankInvariantApprovedProposal, CanonicalProposalPayload},
    schema::{
        BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, DisburseEstateCapability,
        DisburseEstateOperation, EstateCase, EstateCaseIdentityField, ExternalPrincipalMapping,
        Principal,
    },
};

use super::{
    EstateMutationDenial, EstateMutationDenialBinding, EstateMutationResult,
    EstateMutationResultBinding,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisburseEstate {
    action: EstateAction,
    estate: EstateCaseId,
}

impl DisburseEstate {
    pub const fn new(disbursement: EstateDisbursement) -> Self {
        Self {
            action: EstateAction::DisburseEstate(disbursement),
            estate: disbursement.estate,
        }
    }
}

pub struct DisburseEstateMutationBinding;

impl ApplicationMutationBinding<BankSchema> for DisburseEstateMutationBinding {
    type Input = EstateAction;
    type InputBinding = super::super::EstateActionInputBinding;
    type Result = EstateMutationResult;
    type ResultBinding = EstateMutationResultBinding;
    type IdempotencyKey = BankIdempotencyKey;
    type Operation = DisburseEstateOperation;
    type Decision = BankInvariantApprovedProposal;
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

    const IDENTITY: &'static str = "bank.estate.disburse.mutation-binding.v1";
    const HANDLER_IDENTITY: &'static str = "bank.estate.disburse.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "bank.application-mutation-client-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(3, 0, 4, 0, 12, 2),
            ApplicationCandidateResourceCeiling::bounded(32768, 16459),
        );

    fn idempotency_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
        crate::schema::operations::client_key_identity(key)
    }

    fn input_identity(input: &EstateAction) -> [u8; 32] {
        let EstateAction::DisburseEstate(disbursement) = input else {
            return super::invalid_variant_identity("application-disburse-estate", input);
        };
        disbursement_identity(disbursement)
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

fn disbursement_identity(disbursement: &EstateDisbursement) -> [u8; 32] {
    *CanonicalProposalPayload::new("application-disburse-estate")
        .u64("estate", disbursement.estate.get())
        .text(
            "source-account",
            &disbursement.source_account.canonical_text(),
        )
        .text(
            "destination-account",
            &disbursement.destination_account.canonical_text(),
        )
        .u64("beneficiary", disbursement.beneficiary.get())
        .i64("amount-minor", disbursement.amount.minor_units())
        .text(
            "debit-account",
            &disbursement.postings[0].account.canonical_text(),
        )
        .i64("debit-minor", disbursement.postings[0].amount.minor_units())
        .text(
            "credit-account",
            &disbursement.postings[1].account.canonical_text(),
        )
        .i64(
            "credit-minor",
            disbursement.postings[1].amount.minor_units(),
        )
        .derive_identity()
        .bytes()
}

impl ApplicationCapabilityMutationBinding<BankSchema> for DisburseEstateMutationBinding {
    type Capability = DisburseEstateCapability;
}

impl ApplicationMutationIntent<BankSchema> for DisburseEstate {
    type Binding = DisburseEstateMutationBinding;

    fn input(&self) -> &EstateAction {
        &self.action
    }

    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<BankSchema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(EstateCaseIdentityField::reference(), self.estate)
    }
}
