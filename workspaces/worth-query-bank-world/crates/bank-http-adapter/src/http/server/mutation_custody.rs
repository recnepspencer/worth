//! Transport records contain actual native owners, never a replacement phase engine.
#![allow(
    clippy::result_large_err,
    reason = "Close refusal returns the complete native owner without a wrapper allocation"
)]
use super::super::protocol::{BankHttpCommitDescription, BankHttpDenial};
use super::mutation_refusal::source_denial;
use bank_domain::schema::{
    BankSchema, Deposit, FreezeEstateAccount, RestoreFrozenEstateAccountBinding,
    ReverseJournalMutationBinding, SendMoney, Withdraw,
};
use bank_server::{
    BankApplication, DepositAction, EstateAccountFreezeAction, RestoreFrozenAccountAction,
    ReverseJournalAction, SendMoneyAction, WithdrawalAction,
};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::*,
    declaration::application_program::{ApplicationRetainedCommit, ApplicationSourceCompletion},
};

mod source_operation;
use source_operation::custody;

custody!(
    DepositCustody,
    Deposit,
    DepositAction,
    ApplicationRetainedCommit,
    retained
);
custody!(
    WithdrawalCustody,
    Withdraw,
    WithdrawalAction,
    ApplicationRetainedCommit,
    retained
);
custody!(
    SendMoneyCustody,
    SendMoney,
    SendMoneyAction,
    ApplicationRetainedCommit,
    retained
);
custody!(
    CorrectionCustody,
    WorthQueryCapturedMutationInput<BankSchema, ReverseJournalMutationBinding>,
    ReverseJournalAction,
    ApplicationRetainedCommit,
    retained
);
custody!(
    ReapplicationCustody,
    WorthQueryCapturedMutationInput<BankSchema, bank_domain::schema::SendMoneyMutationBinding>,
    SendMoneyAction,
    ApplicationRetainedCommit,
    retained
);

custody!(
    EstateFreezeCustody,
    FreezeEstateAccount,
    EstateAccountFreezeAction,
    ApplicationSourceCompletion,
    source
);
custody!(
    EstateCorrectionCustody,
    WorthQueryCapturedMutationInput<BankSchema, RestoreFrozenEstateAccountBinding>,
    RestoreFrozenAccountAction,
    ApplicationSourceCompletion,
    source
);

pub(super) enum SourceMutationCustody {
    CapabilityRevocation(super::capability_revocation_custody::CapabilityRevocationCustody),
    EstateFreeze(EstateFreezeCustody),
    EstateCorrection(EstateCorrectionCustody),
    Deposit(DepositCustody),
    Withdraw(WithdrawalCustody),
    SendMoney(SendMoneyCustody),
    Correction(CorrectionCustody),
    Reapplication(ReapplicationCustody),
}
pub(super) enum CustodyDescription {
    Pending(
        super::super::protocol::BankHttpMutationPosture,
        Option<super::super::protocol::BankHttpNativeCleanupFailureDescription>,
    ),
    Completed {
        recovered: bool,
        commit: BankHttpCommitDescription,
        source: worth_query_host::facade::primary_graph::WorthQueryCorrectionSourceCoordinates,
    },
    Terminal,
}
impl SourceMutationCustody {
    pub(super) fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
        history: NonZeroUsize,
    ) -> (Self, Option<BankHttpDenial>) {
        match self {
            Self::Reapplication(owner) => {
                let (owner, denial) = owner.advance(request, history);
                (Self::Reapplication(owner), denial)
            }
            Self::CapabilityRevocation(owner) => {
                let (owner, denial) = owner.advance(request);
                (Self::CapabilityRevocation(owner), denial)
            }
            Self::EstateFreeze(owner) => {
                let (owner, denial) = owner.advance(request, history);
                (Self::EstateFreeze(owner), denial)
            }
            Self::EstateCorrection(owner) => {
                let (owner, denial) = owner.advance(request, history);
                (Self::EstateCorrection(owner), denial)
            }
            Self::Correction(owner) => {
                let (owner, denial) = owner.advance(request, history);
                (Self::Correction(owner), denial)
            }
            Self::Deposit(owner) => {
                let (owner, denial) = owner.advance(request, history);
                (Self::Deposit(owner), denial)
            }
            Self::Withdraw(owner) => {
                let (owner, denial) = owner.advance(request, history);
                (Self::Withdraw(owner), denial)
            }
            Self::SendMoney(owner) => {
                let (owner, denial) = owner.advance(request, history);
                (Self::SendMoney(owner), denial)
            }
        }
    }
    pub(super) fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
    ) -> Result<(), (Self, BankHttpDenial)> {
        match self {
            Self::Reapplication(owner) => owner
                .close(request)
                .map_err(|(owner, denial)| (Self::Reapplication(owner), denial)),
            Self::CapabilityRevocation(owner) => owner
                .close(request)
                .map_err(|(owner, denial)| (Self::CapabilityRevocation(owner), denial)),
            Self::EstateFreeze(owner) => owner
                .close(request)
                .map_err(|(owner, denial)| (Self::EstateFreeze(owner), denial)),
            Self::EstateCorrection(owner) => owner
                .close(request)
                .map_err(|(owner, denial)| (Self::EstateCorrection(owner), denial)),
            Self::Correction(owner) => owner
                .close(request)
                .map_err(|(owner, denial)| (Self::Correction(owner), denial)),
            Self::Deposit(owner) => owner
                .close(request)
                .map_err(|(owner, denial)| (Self::Deposit(owner), denial)),
            Self::Withdraw(owner) => owner
                .close(request)
                .map_err(|(owner, denial)| (Self::Withdraw(owner), denial)),
            Self::SendMoney(owner) => owner
                .close(request)
                .map_err(|(owner, denial)| (Self::SendMoney(owner), denial)),
        }
    }
    pub(super) fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
    ) -> Result<WorthQueryOperationCleanupReceipt, BankHttpDenial> {
        match self {
            Self::Reapplication(owner) => owner.release_cleanup(request),
            Self::CapabilityRevocation(owner) => owner.release_cleanup(request),
            Self::EstateFreeze(owner) => owner.release_cleanup(request),
            Self::EstateCorrection(owner) => owner.release_cleanup(request),
            Self::Correction(owner) => owner.release_cleanup(request),
            Self::Deposit(owner) => owner.release_cleanup(request),
            Self::Withdraw(owner) => owner.release_cleanup(request),
            Self::SendMoney(owner) => owner.release_cleanup(request),
        }
    }
    pub(super) fn observation(&self) -> Option<&WorthQueryApplicationReadObservation> {
        match self {
            Self::Reapplication(owner) => owner.observation(),
            Self::CapabilityRevocation(owner) => owner.observation(),
            Self::EstateFreeze(owner) => owner.observation(),
            Self::EstateCorrection(owner) => owner.observation(),
            Self::Correction(owner) => owner.observation(),
            Self::Deposit(owner) => owner.observation(),
            Self::Withdraw(owner) => owner.observation(),
            Self::SendMoney(owner) => owner.observation(),
        }
    }
    pub(super) fn description(&self) -> CustodyDescription {
        match self {
            Self::Reapplication(owner) => owner.description(),
            Self::CapabilityRevocation(owner) => owner.description(),
            Self::EstateFreeze(owner) => owner.description(),
            Self::EstateCorrection(owner) => owner.description(),
            Self::Correction(owner) => owner.description(),
            Self::Deposit(owner) => owner.description(),
            Self::Withdraw(owner) => owner.description(),
            Self::SendMoney(owner) => owner.description(),
        }
    }
}
pub(super) struct SourceMutationRecord {
    pub(super) owner: super::authenticated_owner::BankHttpAuthenticatedOwner,
    pub(super) sequence: u64,
    pub(super) custody: SourceMutationCustody,
    pub(super) _permit: tokio::sync::OwnedSemaphorePermit,
}
pub(super) fn describe_commit(
    receipt: &worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt,
) -> BankHttpCommitDescription {
    let preconditions = receipt.precondition_comparison();
    BankHttpCommitDescription {
        changed_record_count: receipt.changed_record_count(),
        emitted_effect_count: receipt.emitted_effect_count(),
        expected_version_count: preconditions.expected_version_count(),
        expected_fact_count: preconditions.expected_fact_count(),
        provider_work_units: None,
        invariant_work_units: receipt
            .mutation_work()
            .and_then(|work| usize::try_from(work.invariant_work_units()).ok()),
    }
}

pub(super) enum MutationRecord {
    Handoff(super::native_cleanup_custody::HandoffRecord),
    Source(SourceMutationRecord),
    Native(super::native_cleanup_custody::NativeCleanupRecord),
    Estate(
        super::estate_source_custody::EstateSourceRecord,
        tokio::sync::OwnedSemaphorePermit,
    ),
}

impl From<worth_query_host::facade::application_entry::WorthQuerySourceOnlyPosture>
    for super::super::protocol::BankHttpMutationPosture
{
    fn from(
        value: worth_query_host::facade::application_entry::WorthQuerySourceOnlyPosture,
    ) -> Self {
        use worth_query_host::facade::application_entry::WorthQuerySourceOnlyPosture as Native;
        match value {
            Native::Initial => Self::Initial,
            Native::ProductUnpublished => Self::ProductUnpublished,
            Native::RecoveryPerformed => Self::RecoveryPerformed,
            Native::RetainedDeliveryPending => Self::RetainedDeliveryPending,
            Native::ProvedAborted => Self::ProvedAborted,
            Native::RetryableNoEffect => Self::RetryableNoEffect,
            Native::Indeterminate => Self::Indeterminate,
            Native::Terminal => Self::Terminal,
        }
    }
}
