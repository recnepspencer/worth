use worth_query_decl::facade::{
    application_operation::{
        ApplicationCapabilityAdmission, ApplicationCapabilityMutationBinding,
        NoApplicationMutationOutputs,
    },
    worth_query_mutation_binding,
};

use crate::{
    estate::{EstateAction, EstateCaseId, EstateDisbursement},
    proposals::BankInvariantApprovedProposal,
    schema::{
        BankBindingContext, BankSchema, DisburseEstateCapability, DisburseEstateOperation,
        EstateActionInputBinding, EstateCaseIdentityField,
    },
};

use super::{EstateMutationDenialBinding, EstateMutationResultBinding};

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

    const fn shared_action(&self) -> &EstateAction {
        &self.action
    }

    const fn scope_estate(&self) -> EstateCaseId {
        self.estate
    }
}

worth_query_mutation_binding!(
    pub DisburseEstateMutationBinding for EstateAction, schema BankSchema, context BankBindingContext,
    intent DisburseEstate => DisburseEstate::shared_action,
    identity "bank.estate.disburse.mutation-binding.v1",
    input EstateActionInputBinding,
    operation DisburseEstateOperation,
    result EstateMutationResultBinding,
    decision BankInvariantApprovedProposal,
    denial EstateMutationDenialBinding,
    handler identity "bank.estate.disburse.handler.v1",
    admission ApplicationCapabilityAdmission<DisburseEstateCapability>,
    program required,
    outputs NoApplicationMutationOutputs,
    scope EstateCaseIdentityField,
    field EstateCaseIdentityField::at("EstateCaseIdentityField"),
    value DisburseEstate::scope_estate,
);

impl ApplicationCapabilityMutationBinding<BankSchema> for DisburseEstateMutationBinding {
    type Capability = DisburseEstateCapability;
}

impl TryFrom<EstateAction> for DisburseEstate {
    type Error = EstateAction;
    fn try_from(action: EstateAction) -> Result<Self, Self::Error> {
        match action {
            EstateAction::DisburseEstate(disbursement) => Ok(Self::new(disbursement)),
            original => Err(original),
        }
    }
}
