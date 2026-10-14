//! Transport records contain actual native owners, never a replacement phase engine.
use super::super::protocol::{BankHttpCommitDescription, BankHttpDenial};
use super::mutation_refusal::source_denial;
use bank_domain::{
    model::AccountId,
    schema::{BankSchema, Deposit, SendMoney, Withdraw},
};
use bank_server::{BankApplication, DepositAction, SendMoneyAction, WithdrawalAction};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::*, declaration::application_program::ApplicationRetainedCommit,
};

macro_rules! custody {
    ($state:ident, $input:ty, $action:ty) => {
        pub(super) enum $state {
            Active(
                WorthQuerySourceOnlyOperation<
                    BankSchema,
                    $input,
                    BankApplication,
                    $action,
                    ApplicationRetainedCommit,
                >,
            ),
            Completed(
                WorthQueryCompletedSourceOperation<
                    BankSchema,
                    $input,
                    BankApplication,
                    $action,
                    ApplicationRetainedCommit,
                >,
            ),
            Terminal(
                WorthQueryTerminalSourceOperation<
                    BankSchema,
                    $input,
                    BankApplication,
                    $action,
                    ApplicationRetainedCommit,
                >,
            ),
        }
        impl $state {
            fn advance(
                self,
                request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
                history: NonZeroUsize,
            ) -> (Self, Option<BankHttpDenial>) {
                match self {
                    Self::Active(operation) => match operation.advance(request, history) {
                        WorthQuerySourceOnlyAdvance::Progress(operation) => {
                            (Self::Active(operation), None)
                        }
                        WorthQuerySourceOnlyAdvance::Refused { operation, denial } => {
                            (Self::Active(operation), Some(source_denial(denial)))
                        }
                        WorthQuerySourceOnlyAdvance::Completed(operation) => {
                            (Self::Completed(operation), None)
                        }
                        WorthQuerySourceOnlyAdvance::Terminal(operation) => {
                            (Self::Terminal(operation), None)
                        }
                    },
                    owner => (owner, None),
                }
            }
            fn close(
                self,
                request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
            ) -> Result<(), (Self, BankHttpDenial)> {
                match self {
                    Self::Active(operation) => {
                        operation
                            .close(request)
                            .map(|_| ())
                            .map_err(|(owner, denial)| {
                                (
                                    Self::Active(owner),
                                    super::mutation_refusal::close_denial(denial),
                                )
                            })
                    }
                    Self::Completed(operation) => {
                        operation
                            .close(request)
                            .map(|_| ())
                            .map_err(|(owner, denial)| {
                                (
                                    Self::Completed(owner),
                                    super::mutation_refusal::close_denial(denial),
                                )
                            })
                    }
                    Self::Terminal(operation) => {
                        operation
                            .close(request)
                            .map(|_| ())
                            .map_err(|(owner, denial)| {
                                (
                                    Self::Terminal(owner),
                                    super::mutation_refusal::close_denial(denial),
                                )
                            })
                    }
                }
            }
            fn observation(&self) -> Option<&WorthQueryApplicationReadObservation> {
                match self {
                    Self::Completed(operation) => match operation.evidence() {
                        WorthQuerySourceCompletion::Fresh { retained, .. }
                        | WorthQuerySourceCompletion::Recovered { retained, .. } => {
                            retained.as_ref()
                        }
                    },
                    _ => None,
                }
            }
            fn description(&self) -> CustodyDescription {
                match self {
                    Self::Active(operation) => {
                        CustodyDescription::Pending(operation.posture().into())
                    }
                    Self::Terminal(_) => CustodyDescription::Terminal,
                    Self::Completed(operation) => {
                        let (receipt, recovered) = match operation.evidence() {
                            WorthQuerySourceCompletion::Fresh { receipt, .. } => (receipt, false),
                            WorthQuerySourceCompletion::Recovered { receipt, .. } => {
                                (receipt, true)
                            }
                        };
                        CustodyDescription::Completed {
                            recovered,
                            commit: describe_commit(receipt),
                        }
                    }
                }
            }
        }
    };
}
custody!(DepositCustody, Deposit, DepositAction);
custody!(WithdrawalCustody, Withdraw, WithdrawalAction);
custody!(SendMoneyCustody, SendMoney, SendMoneyAction);

pub(super) enum MoneyCustody {
    Deposit(DepositCustody),
    Withdraw(WithdrawalCustody),
    SendMoney(SendMoneyCustody),
}
pub(super) enum CustodyDescription {
    Pending(super::super::protocol::BankHttpMutationPosture),
    Completed {
        recovered: bool,
        commit: BankHttpCommitDescription,
    },
    Terminal,
}
impl MoneyCustody {
    pub(super) fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
        history: NonZeroUsize,
    ) -> (Self, Option<BankHttpDenial>) {
        match self {
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
    pub(super) fn observation(&self) -> Option<&WorthQueryApplicationReadObservation> {
        match self {
            Self::Deposit(owner) => owner.observation(),
            Self::Withdraw(owner) => owner.observation(),
            Self::SendMoney(owner) => owner.observation(),
        }
    }
    pub(super) fn description(&self) -> CustodyDescription {
        match self {
            Self::Deposit(owner) => owner.description(),
            Self::Withdraw(owner) => owner.description(),
            Self::SendMoney(owner) => owner.description(),
        }
    }
}
pub(super) struct MutationRecord {
    pub(super) owner: super::authenticated_owner::BankHttpAuthenticatedOwner,
    pub(super) sequence: u64,
    pub(super) account: AccountId,
    pub(super) custody: MoneyCustody,
    pub(super) _permit: tokio::sync::OwnedSemaphorePermit,
}
fn describe_commit(
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
