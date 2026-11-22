use worth_query_decl::facade::{
    application_operation::{
        ApplicationCapabilityAdmission, ApplicationCapabilityMutationBinding,
        NoApplicationMutationOutputs,
    },
    worth_query_mutation_binding,
};

use crate::{
    estate::{EstateAction, EstateCaseId, LegalAuthorityId, MandatoryReviewId},
    model::BankPrincipalId,
    schema::{
        BankBindingContext, BankSchema, EstateActionInputBinding, EstateCaseIdentityField,
        ReleaseEstateCapability, ReleaseEstateOperation,
    },
};

use super::{EstateMutationDenialBinding, EstateMutationResultBinding};

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

    const fn shared_action(&self) -> &EstateAction {
        &self.action
    }

    const fn scope_estate(&self) -> EstateCaseId {
        self.estate
    }
}

worth_query_mutation_binding!(
    pub ReleaseEstateMutationBinding for EstateAction, schema BankSchema, context BankBindingContext,
    intent ReleaseEstate => ReleaseEstate::shared_action,
    identity "bank.estate.release.mutation-binding.v1",
    input EstateActionInputBinding,
    operation ReleaseEstateOperation,
    result EstateMutationResultBinding,
    decision EstateAction,
    denial EstateMutationDenialBinding,
    handler identity "bank.estate.release.handler.v1",
    admission ApplicationCapabilityAdmission<ReleaseEstateCapability>,
    program required,
    outputs NoApplicationMutationOutputs,
    scope EstateCaseIdentityField,
    field EstateCaseIdentityField::at("EstateCaseIdentityField"),
    value ReleaseEstate::scope_estate,
);

impl ApplicationCapabilityMutationBinding<BankSchema> for ReleaseEstateMutationBinding {
    type Capability = ReleaseEstateCapability;
}

impl TryFrom<EstateAction> for ReleaseEstate {
    type Error = EstateAction;
    fn try_from(action: EstateAction) -> Result<Self, Self::Error> {
        match action {
            EstateAction::ReleaseEstate {
                estate,
                executor,
                authority,
                review,
            } => Ok(Self::new(estate, executor, authority, review)),
            original => Err(original),
        }
    }
}
