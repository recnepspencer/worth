use worth_query_decl::facade::{
    application_operation::{
        ApplicationCapabilityAdmission, ApplicationCapabilityMutationBinding,
        NoApplicationMutationOutputs,
    },
    worth_query_mutation_binding,
};

use crate::{
    estate::{DeathNoticeId, EstateAction, EstateCaseId},
    schema::{
        BankBindingContext, BankSchema, EstateActionInputBinding, EstateCaseIdentityField,
        OpenEstateCaseCapability, OpenEstateCaseOperation,
    },
};

use super::{EstateMutationDenialBinding, EstateMutationResultBinding};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenEstateCase {
    action: EstateAction,
    estate: EstateCaseId,
}

impl OpenEstateCase {
    pub const fn new(estate: EstateCaseId, notice: DeathNoticeId) -> Self {
        Self {
            action: EstateAction::OpenEstateCase { estate, notice },
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
    pub OpenEstateCaseMutationBinding for EstateAction, schema BankSchema, context BankBindingContext,
    intent OpenEstateCase => OpenEstateCase::shared_action,
    identity "bank.estate.open-case.mutation-binding.v1",
    input EstateActionInputBinding,
    operation OpenEstateCaseOperation,
    result EstateMutationResultBinding,
    decision EstateAction,
    denial EstateMutationDenialBinding,
    handler identity "bank.estate.open-case.handler.v1",
    admission ApplicationCapabilityAdmission<OpenEstateCaseCapability>,
    program required,
    outputs NoApplicationMutationOutputs,
    scope EstateCaseIdentityField,
    field EstateCaseIdentityField::at("EstateCaseIdentityField"),
    value OpenEstateCase::scope_estate,
);

impl ApplicationCapabilityMutationBinding<BankSchema> for OpenEstateCaseMutationBinding {
    type Capability = OpenEstateCaseCapability;
}

impl TryFrom<EstateAction> for OpenEstateCase {
    type Error = EstateAction;
    fn try_from(action: EstateAction) -> Result<Self, Self::Error> {
        match action {
            EstateAction::OpenEstateCase { estate, notice } => Ok(Self::new(estate, notice)),
            original => Err(original),
        }
    }
}
