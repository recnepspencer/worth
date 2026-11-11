//! Native source-completion owners for the two estate effects; no host phase engine.
#![allow(
    clippy::result_large_err,
    reason = "Close refusal returns the complete native owner without a wrapper allocation"
)]
use super::super::protocol::{BankHttpDenial, BankHttpMutationOutcome};
use bank_domain::schema::{BankSchema, DisburseEstate, NotifyEstateDeath};
use bank_server::{
    BankApplication, BankClosedSourceCompletion, BankCommitReceipt, DeathNotificationAction,
    EstateDisbursementAction,
};
use worth_query_host::facade::{
    application_entry::*, declaration::application_program::ApplicationSourceCompletion,
};
macro_rules! custody {
    ($name:ident, $input:ty, $action:ty) => {
        pub(super) enum $name {
            Active(
                WorthQuerySourceOnlyOperation<
                    BankSchema,
                    $input,
                    BankApplication,
                    $action,
                    ApplicationSourceCompletion,
                >,
            ),
            Completed(
                WorthQueryCompletedSourceOperation<
                    BankSchema,
                    $input,
                    BankApplication,
                    $action,
                    ApplicationSourceCompletion,
                >,
            ),
            Terminal(
                WorthQueryTerminalSourceOperation<
                    BankSchema,
                    $input,
                    BankApplication,
                    $action,
                    ApplicationSourceCompletion,
                >,
            ),
            Closed(BankClosedSourceCompletion<bank_domain::schema::EstateMutationResult>),
            ClosedWithoutCompletion(WorthQueryClosedOperation),
        }
        impl $name {
            fn advance(
                self,
                request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
            ) -> (Self, Option<BankHttpDenial>) {
                match self {
                    Self::Active(owner) => match owner.advance(request) {
                        WorthQuerySourceOnlyAdvance::Progress(owner) => (Self::Active(owner), None),
                        WorthQuerySourceOnlyAdvance::Refused { operation, denial } => (
                            Self::Active(operation),
                            Some(super::mutation_refusal::source_denial(denial)),
                        ),
                        WorthQuerySourceOnlyAdvance::Completed(owner) => {
                            (Self::Completed(owner), None)
                        }
                        WorthQuerySourceOnlyAdvance::Terminal(owner) => {
                            (Self::Terminal(owner), None)
                        }
                    },
                    owner => (owner, None),
                }
            }
            fn release(
                &mut self,
                request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
            ) -> Result<WorthQueryOperationCleanupReceipt, BankHttpDenial> {
                match self {
                    Self::Active(owner) => owner.release_cleanup(request),
                    Self::Terminal(owner) => owner.release_cleanup(request),
                    _ => {
                        return Err(super::mutation_refusal::denial(
                            super::super::protocol::BankHttpDenialKind::Stale,
                        ))
                    }
                }
                .map_err(super::mutation_refusal::close_denial)
            }
            fn close(
                self,
                request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
            ) -> Result<Self, (Self, BankHttpDenial)> {
                match self {
                    Self::Completed(owner) => owner
                        .close_with_evidence(request)
                        .map(|closed| Self::Closed(BankClosedSourceCompletion::from_query(closed)))
                        .map_err(|(owner, denial)| {
                            (
                                Self::Completed(owner),
                                super::mutation_refusal::close_denial(denial),
                            )
                        }),
                    Self::Active(owner) => owner
                        .close(request)
                        .map(Self::ClosedWithoutCompletion)
                        .map_err(|(owner, denial)| {
                            (
                                Self::Active(owner),
                                super::mutation_refusal::close_denial(denial),
                            )
                        }),
                    Self::Terminal(owner) => owner
                        .close(request)
                        .map(Self::ClosedWithoutCompletion)
                        .map_err(|(owner, denial)| {
                            (
                                Self::Terminal(owner),
                                super::mutation_refusal::close_denial(denial),
                            )
                        }),
                    closed @ (Self::Closed(_) | Self::ClosedWithoutCompletion(_)) => Ok(closed),
                }
            }
            fn take_closed(
                self,
            ) -> Result<
                super::closed_source_evidence::ClosedSourceEvidence<
                    super::closed_source_evidence::SourceResult<$input>,
                >,
                Self,
            > {
                match self {
                    Self::Closed(owner) => {
                        Ok(super::closed_source_evidence::ClosedSourceEvidence::Completion(owner))
                    }
                    Self::ClosedWithoutCompletion(owner) => Ok(
                        super::closed_source_evidence::ClosedSourceEvidence::WithoutCompletion(
                            owner,
                        ),
                    ),
                    owner @ (Self::Active(_) | Self::Completed(_) | Self::Terminal(_)) => {
                        Err(owner)
                    }
                }
            }
            fn live_receipt(&self) -> Option<&BankCommitReceipt> {
                match self {
                    Self::Closed(closed) => closed.live_receipt(),
                    _ => None,
                }
            }
            fn cleanup_failure(
                &self,
            ) -> Option<&worth_query_host::facade::primary_graph::WorthQueryMutationCleanupFailure>
            {
                match self {
                    Self::Active(owner) => owner.managed_cleanup_failure(),
                    Self::Terminal(owner) => owner.managed_cleanup_failure(),
                    _ => None,
                }
            }
            fn describe(
                &self,
                id: &str,
                request_id: &str,
                sequence: u64,
            ) -> BankHttpMutationOutcome {
                let request_id = request_id.into();
                let custody_id = id.into();
                match self {
                    Self::Active(owner) => BankHttpMutationOutcome::Pending {
                        request_id,
                        custody_id,
                        sequence,
                        posture: owner.posture().into(),
                        cleanup: self
                            .cleanup_failure()
                            .map(super::cleanup_description::failure),
                    },
                    Self::Terminal(_) => BankHttpMutationOutcome::Terminal {
                        request_id,
                        custody_id,
                        sequence,
                    },
                    Self::Completed(owner) => {
                        let (receipt, recovered) = match owner.evidence() {
                            WorthQuerySourceCompletion::Recorded { receipt, .. } => {
                                return BankHttpMutationOutcome::Recorded {
                                    request_id,
                                    custody_id,
                                    sequence,
                                    commit: super::mutation_custody::describe_recorded(receipt),
                                }
                            }
                            WorthQuerySourceCompletion::Fresh { receipt, .. } => (receipt, false),
                            WorthQuerySourceCompletion::Recovered { receipt, .. } => {
                                (receipt, true)
                            }
                        };
                        BankHttpMutationOutcome::Completed {
                            request_id,
                            custody_id,
                            sequence,
                            recovered,
                            commit: super::mutation_custody::describe_commit(receipt),
                            source: receipt.correction_source().coordinates().into(),
                        }
                    }
                    Self::Closed(closed) => match closed.live_receipt() {
                        Some(receipt) => BankHttpMutationOutcome::AwaitingRailHandoff {
                            request_id,
                            custody_id,
                            sequence,
                            commit: super::recovery_executor::outcome::commit_description(receipt),
                        },
                        None => BankHttpMutationOutcome::Recorded {
                            request_id,
                            custody_id,
                            sequence,
                            commit: super::mutation_custody::describe_recorded(
                                closed
                                    .recorded_receipt()
                                    .expect("closed completion retains live or recorded evidence"),
                            ),
                        },
                    },
                    Self::ClosedWithoutCompletion(_) => BankHttpMutationOutcome::Closed {
                        request_id,
                        custody_id,
                        sequence,
                    },
                }
            }
        }
    };
}
custody!(
    NotificationCustody,
    NotifyEstateDeath,
    DeathNotificationAction
);
custody!(
    DisbursementCustody,
    DisburseEstate,
    EstateDisbursementAction
);
pub(super) enum EstateSourceCustody {
    Notification(NotificationCustody),
    Disbursement(DisbursementCustody),
}
impl EstateSourceCustody {
    pub(super) fn take_closed(
        self,
    ) -> Result<super::closed_mutation_estate::ClosedMutationEstate, Self> {
        use super::closed_mutation_estate::ClosedMutationEstate;
        match self {
            Self::Notification(owner) => owner
                .take_closed()
                .map(ClosedMutationEstate::Notification)
                .map_err(Self::Notification),
            Self::Disbursement(owner) => owner
                .take_closed()
                .map(ClosedMutationEstate::Disbursement)
                .map_err(Self::Disbursement),
        }
    }

    pub(super) fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
    ) -> (Self, Option<BankHttpDenial>) {
        match self {
            Self::Notification(owner) => {
                let (owner, denial) = owner.advance(request);
                (Self::Notification(owner), denial)
            }
            Self::Disbursement(owner) => {
                let (owner, denial) = owner.advance(request);
                (Self::Disbursement(owner), denial)
            }
        }
    }
    pub(super) fn release(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
    ) -> Result<WorthQueryOperationCleanupReceipt, BankHttpDenial> {
        match self {
            Self::Notification(owner) => owner.release(request),
            Self::Disbursement(owner) => owner.release(request),
        }
    }
    pub(super) fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
    ) -> Result<Self, (Self, BankHttpDenial)> {
        match self {
            Self::Notification(owner) => owner
                .close(request)
                .map(Self::Notification)
                .map_err(|(owner, denial)| (Self::Notification(owner), denial)),
            Self::Disbursement(owner) => owner
                .close(request)
                .map(Self::Disbursement)
                .map_err(|(owner, denial)| (Self::Disbursement(owner), denial)),
        }
    }
    pub(super) fn live_receipt(&self) -> Option<&BankCommitReceipt> {
        match self {
            Self::Notification(owner) => owner.live_receipt(),
            Self::Disbursement(owner) => owner.live_receipt(),
        }
    }
    pub(super) fn describe(
        &self,
        id: &str,
        request_id: &str,
        sequence: u64,
    ) -> BankHttpMutationOutcome {
        match self {
            Self::Notification(owner) => owner.describe(id, request_id, sequence),
            Self::Disbursement(owner) => owner.describe(id, request_id, sequence),
        }
    }
}
pub(super) struct EstateSourceRecord {
    pub(super) owner: super::authenticated_owner::BankHttpAuthenticatedOwner,
    pub(super) sequence: u64,
    pub(super) custody: EstateSourceCustody,
    pub(super) released_cleanup: Option<WorthQueryOperationCleanupReceipt>,
    pub(super) key: bank_domain::proposals::BankIdempotencyKey,
    pub(super) action: bank_domain::estate::EstateAction,
    pub(super) registry:
        std::sync::Arc<tokio::sync::Mutex<super::recovery_registry::BankHttpRecoveryRegistry>>,
}
