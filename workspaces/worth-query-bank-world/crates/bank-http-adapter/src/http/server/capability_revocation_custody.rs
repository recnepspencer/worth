//! HTTP custody contains the original native capability command through lawful close.
#![allow(
    clippy::result_large_err,
    reason = "Refusal returns the original native owner"
)]
use super::mutation_custody::{describe_commit, CustodyDescription};
use crate::BankHttpDenial;
use bank_domain::schema::BankSchema;
use bank_server::{BankApplication, BankEstateProgressionDenial, BankRevocationCommand};
use worth_query_host::facade::application_entry::*;

mod denial;
pub(super) use denial::native_denial;
type NativeDenial = WorthQueryApplicationCapabilityRevocationDenial<BankEstateProgressionDenial>;

pub(super) enum CapabilityRevocationCustody {
    Active(BankRevocationCommand<'static>),
    Completed(
        WorthQueryCompletedCapabilityCommand<'static, BankSchema, BankApplication, NativeDenial>,
    ),
    Terminal(
        WorthQueryTerminalCapabilityCommand<'static, BankSchema, BankApplication, NativeDenial>,
    ),
    Closed(bank_server::BankClosedCapabilityCommand),
    ClosedWithoutCompletion(WorthQueryClosedOperation),
}

impl CapabilityRevocationCustody {
    pub(super) fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
    ) -> (Self, Option<BankHttpDenial>) {
        match self {
            Self::Active(command) => match command.advance(request) {
                WorthQueryCapabilityCommandAdvance::Progress(command) => {
                    (Self::Active(command), None)
                }
                WorthQueryCapabilityCommandAdvance::Refused { command, denial } => {
                    (Self::Active(command), Some(denial::command_denial(denial)))
                }
                WorthQueryCapabilityCommandAdvance::Completed(command) => {
                    (Self::Completed(command), None)
                }
                WorthQueryCapabilityCommandAdvance::Terminal(command) => {
                    (Self::Terminal(command), None)
                }
            },
            owner => (owner, None),
        }
    }

    pub(super) fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
    ) -> Result<Self, (Self, BankHttpDenial)> {
        match self {
            Self::Active(command) => command
                .close(request)
                .map(Self::ClosedWithoutCompletion)
                .map_err(|(owner, cause)| (Self::Active(owner), denial::command_denial(cause))),
            Self::Completed(command) => command
                .close_with_evidence(request)
                .map(|closed| {
                    Self::Closed(bank_server::BankClosedCapabilityCommand::from_query(closed))
                })
                .map_err(|(owner, cause)| (Self::Completed(owner), denial::command_denial(cause))),
            Self::Terminal(command) => command
                .close(request)
                .map(Self::ClosedWithoutCompletion)
                .map_err(|(owner, cause)| (Self::Terminal(owner), denial::command_denial(cause))),
            closed @ (Self::Closed(_) | Self::ClosedWithoutCompletion(_)) => Ok(closed),
        }
    }

    pub(super) fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
    ) -> Result<WorthQueryOperationCleanupReceipt, BankHttpDenial> {
        match self {
            Self::Active(command) => command.release_cleanup(request),
            Self::Completed(command) => command.release_cleanup(request),
            Self::Terminal(command) => command.release_cleanup(request),
            Self::Closed(_) | Self::ClosedWithoutCompletion(_) => {
                return Err(super::mutation_refusal::denial(
                    super::super::protocol::BankHttpDenialKind::Stale,
                ))
            }
        }
        .map_err(denial::command_denial)
    }

    pub(super) fn take_closed(
        self,
    ) -> Result<super::closed_source_evidence::ClosedCapabilityEvidence, Self> {
        match self {
            Self::Closed(owner) => {
                Ok(super::closed_source_evidence::ClosedCapabilityEvidence::Completion(owner))
            }
            Self::ClosedWithoutCompletion(owner) => Ok(
                super::closed_source_evidence::ClosedCapabilityEvidence::WithoutCompletion(owner),
            ),
            owner @ (Self::Active(_) | Self::Completed(_) | Self::Terminal(_)) => Err(owner),
        }
    }
    pub(super) fn observation(&self) -> Option<&WorthQueryApplicationReadObservation> {
        None
    }

    pub(super) fn description(&self) -> CustodyDescription {
        match self {
            Self::Active(command) => CustodyDescription::Pending(
                command.posture().into(),
                command
                    .managed_cleanup_failure()
                    .map(super::cleanup_description::failure),
            ),
            Self::Terminal(_) => CustodyDescription::Terminal,
            Self::Closed(_) | Self::ClosedWithoutCompletion(_) => CustodyDescription::Closed,
            Self::Completed(command) => {
                let evidence = command.evidence();
                let Some(receipt) = evidence.receipt() else {
                    return CustodyDescription::Recorded(
                        super::mutation_custody::describe_recorded(
                            evidence
                                .recovered_receipt()
                                .expect("completion owns live or recorded receipt"),
                        ),
                    );
                };
                CustodyDescription::Completed {
                    recovered: matches!(
                        evidence,
                        WorthQueryCapabilityCommandCompletion::Recovered { .. }
                    ),
                    commit: describe_commit(receipt),
                    source: receipt.correction_source().coordinates(),
                }
            }
        }
    }
}
