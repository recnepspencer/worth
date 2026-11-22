use worth_query_decl::facade::{
    application_operation::{
        ApplicationCapabilityAdmission, ApplicationCapabilityMutationBinding,
        NoApplicationMutationOutputs,
    },
    worth_query_mutation_binding,
};

use crate::{
    estate::{DeathNoticeId, EstateAction, EstateCaseId},
    model::BankPrincipalId,
    schema::{
        BankBindingContext, BankSchema, EstateActionInputBinding, EstateCaseIdentityField,
        RetransmitDeathNoticeEstateCapability, RetransmitDeathNoticeEstateOperation,
    },
};

use super::{EstateMutationDenialBinding, EstateMutationResultBinding};

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

    const fn shared_action(&self) -> &EstateAction {
        &self.action
    }

    const fn scope_estate(&self) -> EstateCaseId {
        self.estate
    }
}

worth_query_mutation_binding!(
    pub RetransmitEstateDeathNoticeMutationBinding for EstateAction, schema BankSchema, context BankBindingContext,
    intent RetransmitEstateDeathNotice => RetransmitEstateDeathNotice::shared_action,
    identity "bank.estate.retransmit-death-notice.mutation-binding.v1",
    input EstateActionInputBinding,
    operation RetransmitDeathNoticeEstateOperation,
    result EstateMutationResultBinding,
    decision EstateAction,
    denial EstateMutationDenialBinding,
    handler identity "bank.estate.retransmit-death-notice.handler.v1",
    admission ApplicationCapabilityAdmission<RetransmitDeathNoticeEstateCapability>,
    program required,
    outputs NoApplicationMutationOutputs,
    scope EstateCaseIdentityField,
    field EstateCaseIdentityField::at("EstateCaseIdentityField"),
    value RetransmitEstateDeathNotice::scope_estate,
);

impl ApplicationCapabilityMutationBinding<BankSchema>
    for RetransmitEstateDeathNoticeMutationBinding
{
    type Capability = RetransmitDeathNoticeEstateCapability;
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
