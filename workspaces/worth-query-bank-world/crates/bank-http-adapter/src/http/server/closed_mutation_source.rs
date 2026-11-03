//! Finite closed mutation families carry no active command or execution backend.
use super::closed_source_evidence::{ClosedCapabilityEvidence, ClosedSourceEvidence, SourceResult};
use bank_domain::schema::{
    BankSchema, Deposit, FreezeEstateAccount, RestoreFrozenEstateAccountBinding,
    ReverseJournalMutationBinding, SendMoney, SendMoneyMutationBinding, Withdraw,
};
use worth_query_host::facade::application_entry::{
    WorthQueryCapturedMutationInput, WorthQueryClosedOperation,
};
pub(super) enum ClosedMutationSource {
    CapabilityRevocation(ClosedCapabilityEvidence),
    EstateFreeze(ClosedSourceEvidence<SourceResult<FreezeEstateAccount>>),
    EstateCorrection(
        ClosedSourceEvidence<
            SourceResult<
                WorthQueryCapturedMutationInput<BankSchema, RestoreFrozenEstateAccountBinding>,
            >,
        >,
    ),
    Deposit(ClosedSourceEvidence<SourceResult<Deposit>>),
    Withdraw(ClosedSourceEvidence<SourceResult<Withdraw>>),
    SendMoney(ClosedSourceEvidence<SourceResult<SendMoney>>),
    Correction(
        ClosedSourceEvidence<
            SourceResult<
                WorthQueryCapturedMutationInput<BankSchema, ReverseJournalMutationBinding>,
            >,
        >,
    ),
    Reapplication(
        ClosedSourceEvidence<
            SourceResult<WorthQueryCapturedMutationInput<BankSchema, SendMoneyMutationBinding>>,
        >,
    ),
}
impl ClosedMutationSource {
    pub(super) fn closed(&self) -> &WorthQueryClosedOperation {
        match self {
            Self::CapabilityRevocation(owner) => owner.closed(),
            Self::EstateFreeze(owner) => owner.closed(),
            Self::EstateCorrection(owner) => owner.closed(),
            Self::Deposit(owner) => owner.closed(),
            Self::Withdraw(owner) => owner.closed(),
            Self::SendMoney(owner) => owner.closed(),
            Self::Correction(owner) => owner.closed(),
            Self::Reapplication(owner) => owner.closed(),
        }
    }
    pub(super) fn closed_mut(&mut self) -> &mut WorthQueryClosedOperation {
        match self {
            Self::CapabilityRevocation(owner) => owner.closed_mut(),
            Self::EstateFreeze(owner) => owner.closed_mut(),
            Self::EstateCorrection(owner) => owner.closed_mut(),
            Self::Deposit(owner) => owner.closed_mut(),
            Self::Withdraw(owner) => owner.closed_mut(),
            Self::SendMoney(owner) => owner.closed_mut(),
            Self::Correction(owner) => owner.closed_mut(),
            Self::Reapplication(owner) => owner.closed_mut(),
        }
    }
}
