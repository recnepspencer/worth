use worth_query_decl::facade::{
    application_operation::{
        ApplicationCapabilityAdmission, ApplicationCapabilityMutationBinding,
        NoApplicationMutationOutputs,
    },
    worth_query_mutation_binding,
};

use crate::{
    estate::{EstateAction, EstateCaseId, LegalAuthorityId},
    model::BankPrincipalId,
    schema::{
        BankBindingContext, BankSchema, EstateActionInputBinding, EstateCaseIdentityField,
        RecognizeEstateExecutorCapability, RecognizeEstateExecutorOperation,
    },
};

use super::{EstateMutationDenialBinding, EstateMutationResultBinding};

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

    const fn shared_action(&self) -> &EstateAction {
        &self.action
    }

    const fn scope_estate(&self) -> EstateCaseId {
        self.estate
    }
}

worth_query_mutation_binding!(
    pub RecognizeEstateExecutorMutationBinding for EstateAction, schema BankSchema, context BankBindingContext,
    intent RecognizeEstateExecutor => RecognizeEstateExecutor::shared_action,
    identity "bank.estate.recognize-executor.mutation-binding.v1",
    input EstateActionInputBinding,
    operation RecognizeEstateExecutorOperation,
    result EstateMutationResultBinding,
    decision EstateAction,
    denial EstateMutationDenialBinding,
    handler identity "bank.estate.recognize-executor.handler.v1",
    admission ApplicationCapabilityAdmission<RecognizeEstateExecutorCapability>,
    program required,
    outputs NoApplicationMutationOutputs,
    scope EstateCaseIdentityField,
    field EstateCaseIdentityField::at("EstateCaseIdentityField"),
    value RecognizeEstateExecutor::scope_estate,
);

impl ApplicationCapabilityMutationBinding<BankSchema> for RecognizeEstateExecutorMutationBinding {
    type Capability = RecognizeEstateExecutorCapability;
}

impl TryFrom<EstateAction> for RecognizeEstateExecutor {
    type Error = EstateAction;
    fn try_from(action: EstateAction) -> Result<Self, Self::Error> {
        match action {
            EstateAction::RecognizeExecutor {
                estate,
                executor,
                authority,
            } => Ok(Self::new(estate, executor, authority)),
            original => Err(original),
        }
    }
}
