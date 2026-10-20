use worth_query_decl::facade::application_operation::ApplicationCandidatePolicy;
use worth_query_decl::facade::application_operation::ApplicationCandidateRequirements;
use worth_query_decl::facade::{
    application_operation::{
        ApplicationCandidateCardinalityCeiling, ApplicationCandidateResourceCeiling,
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationFieldScope, ApplicationMutationIntent, NoApplicationMutationOutputs,
        NoApplicationMutationSource,
    },
    application_schema::{
        ApplicationFieldRef, ApplicationPrincipalBindingRef, EqualityPredicate, NoApplicationUnit,
        ReadOnly,
    },
};

use crate::{
    estate::{DeathNoticeId, EstateAction, EstateCaseId},
    model::BankPrincipalId,
    proposals::BankIdempotencyKey,
    schema::{
        BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, EstateCase,
        EstateCaseIdentityField, ExternalPrincipalMapping, Principal,
        RetransmitDeathNoticeEstateCapability, RetransmitDeathNoticeEstateOperation,
    },
};

use super::{
    EstateMutationDenial, EstateMutationDenialBinding, EstateMutationResult,
    EstateMutationResultBinding,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetransmitEstateDeathNotice {
    action: EstateAction,
    estate: EstateCaseId,
}

impl RetransmitEstateDeathNotice {
    pub const fn new(
        estate: EstateCaseId,
        notice: DeathNoticeId,
        subject: BankPrincipalId,
    ) -> Self {
        Self {
            action: EstateAction::RetransmitDeathNotice {
                estate,
                notice,
                subject,
            },
            estate,
        }
    }
}

pub struct RetransmitEstateDeathNoticeMutationBinding;

impl ApplicationMutationBinding<BankSchema> for RetransmitEstateDeathNoticeMutationBinding {
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = EstateAction;
    type InputBinding = super::super::EstateActionInputBinding;
    type Result = EstateMutationResult;
    type ResultBinding = EstateMutationResultBinding;
    type IdempotencyKey = BankIdempotencyKey;
    type Operation = RetransmitDeathNoticeEstateOperation;
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

    const IDENTITY: &'static str = "bank.estate.retransmit-death-notice.mutation-binding.v1";
    const HANDLER_IDENTITY: &'static str = "bank.estate.retransmit-death-notice.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "bank.application-mutation-client-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidatePolicy =
        ApplicationCandidatePolicy::finite(ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 0, 1),
            ApplicationCandidateResourceCeiling::representation_bytes(32768),
        ));

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

impl ApplicationCapabilityMutationBinding<BankSchema>
    for RetransmitEstateDeathNoticeMutationBinding
{
    type Capability = RetransmitDeathNoticeEstateCapability;
}

impl ApplicationMutationIntent<BankSchema> for RetransmitEstateDeathNotice {
    type Binding = RetransmitEstateDeathNoticeMutationBinding;

    fn input(&self) -> &EstateAction {
        &self.action
    }

    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<BankSchema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(EstateCaseIdentityField::reference(), self.estate)
    }
}

impl
    worth_query_decl::facade::application_operation::ApplicationMutationAdmissionBinding<BankSchema>
    for RetransmitEstateDeathNoticeMutationBinding
{
    type Admission =
        worth_query_decl::facade::application_operation::ApplicationCapabilityAdmission<
            RetransmitDeathNoticeEstateCapability,
        >;
}

impl TryFrom<EstateAction> for RetransmitEstateDeathNotice {
    type Error = EstateAction;
    fn try_from(action: EstateAction) -> Result<Self, Self::Error> {
        match action {
            EstateAction::RetransmitDeathNotice {
                estate,
                notice,
                subject,
            } => Ok(Self::new(estate, notice, subject)),
            original => Err(original),
        }
    }
}
