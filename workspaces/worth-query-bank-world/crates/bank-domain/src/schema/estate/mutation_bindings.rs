use worth_query_decl::facade::{
    application_operation::{
        ApplicationCapabilityAdmission, ApplicationCapabilityMutationBinding,
        ApplicationMutationBinding, ApplicationMutationFieldScope, ApplicationMutationInputScope,
        NoApplicationMutationOutputs,
    },
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::{
    estate::{DeathNoticeId, EstateAction, EstateCaseId},
    model::{AccountId, BankPrincipalId},
};

use crate::schema::{
    BankBindingContext, BankSchema, EstateActionInputBinding, EstateCaseIdentityField,
    FreezeEstateAccountCapability, FreezeEstateAccountOperation, NotifyDeathEstateCapability,
    NotifyDeathEstateOperation,
};

mod open_estate_case;
pub use open_estate_case::*;
mod recognize_executor;
pub use recognize_executor::*;
mod release_estate;
pub use release_estate::*;
mod disburse_estate;
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

    const fn shared_action(&self) -> &EstateAction {
        &self.action
    }

    const fn scope_estate(&self) -> EstateCaseId {
        self.estate
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

    const fn shared_action(&self) -> &EstateAction {
        &self.action
    }

    const fn scope_estate(&self) -> EstateCaseId {
        self.estate
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

worth_query_mutation_binding!(
    pub NotifyEstateDeathMutationBinding for EstateAction, schema BankSchema, context BankBindingContext,
    intent NotifyEstateDeath => NotifyEstateDeath::shared_action,
    identity "bank.estate.notify-death.mutation-binding.v1",
    input EstateActionInputBinding,
    operation NotifyDeathEstateOperation,
    result EstateMutationResultBinding,
    decision EstateAction,
    denial EstateMutationDenialBinding,
    handler identity "bank.estate.notify-death.handler.v1",
    admission ApplicationCapabilityAdmission<NotifyDeathEstateCapability>,
    program required,
    outputs NoApplicationMutationOutputs,
    scope EstateCaseIdentityField,
    field EstateCaseIdentityField::at("EstateCaseIdentityField"),
    value NotifyEstateDeath::scope_estate,
);

impl ApplicationCapabilityMutationBinding<BankSchema> for NotifyEstateDeathMutationBinding {
    type Capability = NotifyDeathEstateCapability;
}

worth_query_mutation_binding!(
    pub FreezeEstateAccountMutationBinding for EstateAction, schema BankSchema, context BankBindingContext,
    intent FreezeEstateAccount => FreezeEstateAccount::shared_action,
    identity "bank.estate.freeze-account.mutation-binding.v1",
    input EstateActionInputBinding,
    operation FreezeEstateAccountOperation,
    result EstateMutationResultBinding,
    decision EstateAction,
    denial EstateMutationDenialBinding,
    handler identity "bank.estate.freeze-account.handler.v1",
    admission ApplicationCapabilityAdmission<FreezeEstateAccountCapability>,
    program required,
    outputs NoApplicationMutationOutputs,
    scope EstateCaseIdentityField,
    field EstateCaseIdentityField::at("EstateCaseIdentityField"),
    value FreezeEstateAccount::scope_estate,
);

impl ApplicationCapabilityMutationBinding<BankSchema> for FreezeEstateAccountMutationBinding {
    type Capability = FreezeEstateAccountCapability;
}

impl ApplicationMutationInputScope<BankSchema, FreezeEstateAccountMutationBinding>
    for EstateAction
{
    fn scope_for_binding(
        &self,
    ) -> <FreezeEstateAccountMutationBinding as ApplicationMutationBinding<BankSchema>>::ScopeBinding
    {
        ApplicationMutationFieldScope::new(
            EstateCaseIdentityField::at("EstateCaseIdentityField"),
            self.estate()
                .expect("every estate action variant carries its estate"),
        )
    }
}

impl TryFrom<EstateAction> for NotifyEstateDeath {
    type Error = EstateAction;
    fn try_from(action: EstateAction) -> Result<Self, Self::Error> {
        match action {
            EstateAction::NotifyDeath {
                estate,
                notice,
                subject,
            } => Ok(Self::new(estate, notice, subject)),
            original => Err(original),
        }
    }
}

impl TryFrom<EstateAction> for FreezeEstateAccount {
    type Error = EstateAction;
    fn try_from(action: EstateAction) -> Result<Self, Self::Error> {
        match action {
            EstateAction::FreezeAccount { estate, account } => Ok(Self::new(estate, account)),
            original => Err(original),
        }
    }
}
