//! Bank-owned classifications for every estate elevation transition.

use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDeferred, WorthQueryElevationApprovalOutcome,
    WorthQueryElevationCloseOutcome, WorthQueryElevationRequestOutcome,
    WorthQueryMandatoryReviewOutcome, WorthQueryProductStaleApplication,
    WorthQueryProductUnpublishedApplication, WorthQueryRecoveredApplicationCommitReceipt,
};
use worth_query_host::facade::product::WorthQueryApplicationNoEffectCause;

use super::{
    BankApprovedEstateElevation, BankEstateMandatoryReview, BankRequestedEstateElevation,
    BankReviewedEstateElevation,
};
use crate::operation_commit::{denial_kind, denial_stage};
use crate::{BankCommitDenialKind, BankCommitDenialStage};

#[derive(Debug)]
pub enum BankEstateElevationRequestOutcome {
    ProductPreparationPending(
        worth_query_host::facade::primary_graph::WorthQueryProductPreparationPending,
    ),
    ProductPreparationRefused(
        worth_query_host::facade::primary_graph::WorthQueryProductPreparationRefused,
    ),
    CleanupRequired(crate::BankManagedCleanup),
    ProductStale(WorthQueryProductStaleApplication),
    ProductUnpublished(WorthQueryProductUnpublishedApplication),
    NoEffect(WorthQueryApplicationNoEffectCause),
    Requested(BankRequestedEstateElevation),
    RecoveredCommitted(WorthQueryRecoveredApplicationCommitReceipt),
    Stale {
        stale_fact_count: usize,
    },
    Cancelled,
    TimedOut,
    Denied {
        kind: BankCommitDenialKind,
        stage: BankCommitDenialStage,
    },
    Aborted,
    Deferred(WorthQueryApplicationCommitDeferred),
    Indeterminate,
}

#[derive(Debug)]
pub enum BankEstateElevationApprovalOutcome {
    ProductPreparationPending(
        worth_query_host::facade::primary_graph::WorthQueryProductPreparationPending,
    ),
    ProductPreparationRefused(
        worth_query_host::facade::primary_graph::WorthQueryProductPreparationRefused,
    ),
    CleanupRequired(crate::BankManagedCleanup),
    ProductStale(
        WorthQueryProductStaleApplication,
        BankRequestedEstateElevation,
    ),
    ProductUnpublished(WorthQueryProductUnpublishedApplication),
    NoEffect(
        WorthQueryApplicationNoEffectCause,
        BankRequestedEstateElevation,
    ),
    Approved(BankApprovedEstateElevation),
    RecoveredCommitted {
        receipt: WorthQueryRecoveredApplicationCommitReceipt,
        requested: BankRequestedEstateElevation,
    },
    Stale {
        stale_fact_count: usize,
        requested: BankRequestedEstateElevation,
    },
    Cancelled(BankRequestedEstateElevation),
    TimedOut,
    Denied {
        kind: BankCommitDenialKind,
        stage: BankCommitDenialStage,
        requested: BankRequestedEstateElevation,
    },
    Aborted(BankRequestedEstateElevation),
    Deferred(WorthQueryApplicationCommitDeferred),
    Indeterminate,
}

#[derive(Debug)]
pub enum BankEstateElevationCloseOutcome {
    ProductPreparationPending(
        worth_query_host::facade::primary_graph::WorthQueryProductPreparationPending,
    ),
    ProductPreparationRefused(
        worth_query_host::facade::primary_graph::WorthQueryProductPreparationRefused,
    ),
    CleanupRequired(crate::BankManagedCleanup),
    ProductStale(
        WorthQueryProductStaleApplication,
        BankApprovedEstateElevation,
    ),
    ProductUnpublished(WorthQueryProductUnpublishedApplication),
    NoEffect(
        WorthQueryApplicationNoEffectCause,
        BankApprovedEstateElevation,
    ),
    Closed(BankEstateMandatoryReview),
    RecoveredCommitted {
        receipt: WorthQueryRecoveredApplicationCommitReceipt,
        approved: BankApprovedEstateElevation,
    },
    Stale {
        stale_fact_count: usize,
        approved: BankApprovedEstateElevation,
    },
    Cancelled(BankApprovedEstateElevation),
    TimedOut,
    Denied {
        kind: BankCommitDenialKind,
        stage: BankCommitDenialStage,
        approved: BankApprovedEstateElevation,
    },
    Aborted(BankApprovedEstateElevation),
    Deferred(WorthQueryApplicationCommitDeferred),
    Indeterminate,
}

#[derive(Debug)]
pub enum BankEstateMandatoryReviewOutcome {
    ProductPreparationPending(
        worth_query_host::facade::primary_graph::WorthQueryProductPreparationPending,
    ),
    ProductPreparationRefused(
        worth_query_host::facade::primary_graph::WorthQueryProductPreparationRefused,
    ),
    CleanupRequired(crate::BankManagedCleanup),
    ProductStale(WorthQueryProductStaleApplication, BankEstateMandatoryReview),
    ProductUnpublished(WorthQueryProductUnpublishedApplication),
    NoEffect(
        WorthQueryApplicationNoEffectCause,
        BankEstateMandatoryReview,
    ),
    Reviewed(BankReviewedEstateElevation),
    RecoveredCommitted {
        receipt: WorthQueryRecoveredApplicationCommitReceipt,
        mandatory: BankEstateMandatoryReview,
    },
    Stale {
        stale_fact_count: usize,
        mandatory: BankEstateMandatoryReview,
    },
    Cancelled(BankEstateMandatoryReview),
    TimedOut,
    Denied {
        kind: BankCommitDenialKind,
        stage: BankCommitDenialStage,
        mandatory: BankEstateMandatoryReview,
    },
    Aborted(BankEstateMandatoryReview),
    Deferred(WorthQueryApplicationCommitDeferred),
    Indeterminate,
}

impl BankEstateElevationRequestOutcome {
    pub(crate) fn from_query(outcome: WorthQueryElevationRequestOutcome) -> Self {
        match outcome {
            WorthQueryElevationRequestOutcome::ProductPreparationPending(pending) => {
                Self::ProductPreparationPending(pending)
            }
            WorthQueryElevationRequestOutcome::ProductPreparationRefused(refused) => {
                Self::ProductPreparationRefused(refused)
            }
            WorthQueryElevationRequestOutcome::CleanupRequired(owner) => {
                Self::CleanupRequired(owner.into())
            }
            WorthQueryElevationRequestOutcome::ProductStale(stale) => Self::ProductStale(stale),
            WorthQueryElevationRequestOutcome::ProductUnpublished(unpublished) => {
                Self::ProductUnpublished(unpublished)
            }
            WorthQueryElevationRequestOutcome::NoEffect(no_effect) => {
                Self::NoEffect(no_effect.cause())
            }
            WorthQueryElevationRequestOutcome::Requested(value) => {
                Self::Requested(BankRequestedEstateElevation::from_query(value))
            }
            WorthQueryElevationRequestOutcome::RecoveredCommitted(value) => {
                Self::RecoveredCommitted(value)
            }
            WorthQueryElevationRequestOutcome::Stale(stale) => Self::Stale {
                stale_fact_count: stale.stale_fact_count(),
            },
            WorthQueryElevationRequestOutcome::Cancelled => Self::Cancelled,
            WorthQueryElevationRequestOutcome::TimedOut => Self::TimedOut,
            WorthQueryElevationRequestOutcome::Denied(denial) => Self::Denied {
                kind: denial_kind(denial.kind()),
                stage: denial_stage(denial.stage()),
            },
            WorthQueryElevationRequestOutcome::Aborted => Self::Aborted,
            WorthQueryElevationRequestOutcome::Deferred(deferred) => Self::Deferred(deferred),
            WorthQueryElevationRequestOutcome::Indeterminate => Self::Indeterminate,
        }
    }
}

impl BankEstateElevationApprovalOutcome {
    pub(crate) fn from_query(outcome: WorthQueryElevationApprovalOutcome) -> Self {
        match outcome {
            WorthQueryElevationApprovalOutcome::ProductPreparationPending(pending) => {
                Self::ProductPreparationPending(pending)
            }
            WorthQueryElevationApprovalOutcome::ProductPreparationRefused(refused) => {
                Self::ProductPreparationRefused(refused)
            }
            WorthQueryElevationApprovalOutcome::CleanupRequired(owner) => {
                Self::CleanupRequired(owner.into())
            }
            WorthQueryElevationApprovalOutcome::ProductStale(stale, requested) => {
                Self::ProductStale(stale, BankRequestedEstateElevation::from_query(requested))
            }
            WorthQueryElevationApprovalOutcome::ProductUnpublished(unpublished) => {
                Self::ProductUnpublished(unpublished)
            }
            WorthQueryElevationApprovalOutcome::NoEffect(no_effect, requested) => Self::NoEffect(
                no_effect.cause(),
                BankRequestedEstateElevation::from_query(requested),
            ),
            WorthQueryElevationApprovalOutcome::Approved(value) => {
                Self::Approved(BankApprovedEstateElevation::from_query(value))
            }
            WorthQueryElevationApprovalOutcome::RecoveredCommitted { receipt, requested } => {
                Self::RecoveredCommitted {
                    receipt,
                    requested: BankRequestedEstateElevation::from_query(requested),
                }
            }
            WorthQueryElevationApprovalOutcome::Stale(stale, requested) => Self::Stale {
                stale_fact_count: stale.stale_fact_count(),
                requested: BankRequestedEstateElevation::from_query(requested),
            },
            WorthQueryElevationApprovalOutcome::Cancelled(requested) => {
                Self::Cancelled(BankRequestedEstateElevation::from_query(requested))
            }
            WorthQueryElevationApprovalOutcome::TimedOut => Self::TimedOut,
            WorthQueryElevationApprovalOutcome::Denied(denial, requested) => Self::Denied {
                kind: denial_kind(denial.kind()),
                stage: denial_stage(denial.stage()),
                requested: BankRequestedEstateElevation::from_query(requested),
            },
            WorthQueryElevationApprovalOutcome::Aborted(requested) => {
                Self::Aborted(BankRequestedEstateElevation::from_query(requested))
            }
            WorthQueryElevationApprovalOutcome::Deferred(deferred) => Self::Deferred(deferred),
            WorthQueryElevationApprovalOutcome::Indeterminate => Self::Indeterminate,
        }
    }
}

impl BankEstateElevationCloseOutcome {
    pub(crate) fn from_query(outcome: WorthQueryElevationCloseOutcome) -> Self {
        match outcome {
            WorthQueryElevationCloseOutcome::ProductPreparationPending(pending) => {
                Self::ProductPreparationPending(pending)
            }
            WorthQueryElevationCloseOutcome::ProductPreparationRefused(refused) => {
                Self::ProductPreparationRefused(refused)
            }
            WorthQueryElevationCloseOutcome::CleanupRequired(owner) => {
                Self::CleanupRequired(owner.into())
            }
            WorthQueryElevationCloseOutcome::ProductStale(stale, approved) => {
                Self::ProductStale(stale, BankApprovedEstateElevation::from_query(approved))
            }
            WorthQueryElevationCloseOutcome::ProductUnpublished(unpublished) => {
                Self::ProductUnpublished(unpublished)
            }
            WorthQueryElevationCloseOutcome::NoEffect(no_effect, approved) => Self::NoEffect(
                no_effect.cause(),
                BankApprovedEstateElevation::from_query(approved),
            ),
            WorthQueryElevationCloseOutcome::Closed(value) => {
                Self::Closed(BankEstateMandatoryReview::from_query(value))
            }
            WorthQueryElevationCloseOutcome::RecoveredCommitted { receipt, approved } => {
                Self::RecoveredCommitted {
                    receipt,
                    approved: BankApprovedEstateElevation::from_query(approved),
                }
            }
            WorthQueryElevationCloseOutcome::Stale(stale, approved) => Self::Stale {
                stale_fact_count: stale.stale_fact_count(),
                approved: BankApprovedEstateElevation::from_query(approved),
            },
            WorthQueryElevationCloseOutcome::Cancelled(approved) => {
                Self::Cancelled(BankApprovedEstateElevation::from_query(approved))
            }
            WorthQueryElevationCloseOutcome::TimedOut => Self::TimedOut,
            WorthQueryElevationCloseOutcome::Denied(denial, approved) => Self::Denied {
                kind: denial_kind(denial.kind()),
                stage: denial_stage(denial.stage()),
                approved: BankApprovedEstateElevation::from_query(approved),
            },
            WorthQueryElevationCloseOutcome::Aborted(approved) => {
                Self::Aborted(BankApprovedEstateElevation::from_query(approved))
            }
            WorthQueryElevationCloseOutcome::Deferred(deferred) => Self::Deferred(deferred),
            WorthQueryElevationCloseOutcome::Indeterminate => Self::Indeterminate,
        }
    }
}

impl BankEstateMandatoryReviewOutcome {
    pub(crate) fn from_query(outcome: WorthQueryMandatoryReviewOutcome) -> Self {
        match outcome {
            WorthQueryMandatoryReviewOutcome::ProductPreparationPending(pending) => {
                Self::ProductPreparationPending(pending)
            }
            WorthQueryMandatoryReviewOutcome::ProductPreparationRefused(refused) => {
                Self::ProductPreparationRefused(refused)
            }
            WorthQueryMandatoryReviewOutcome::CleanupRequired(owner) => {
                Self::CleanupRequired(owner.into())
            }
            WorthQueryMandatoryReviewOutcome::ProductStale(stale, mandatory) => {
                Self::ProductStale(stale, BankEstateMandatoryReview::from_query(mandatory))
            }
            WorthQueryMandatoryReviewOutcome::ProductUnpublished(unpublished) => {
                Self::ProductUnpublished(unpublished)
            }
            WorthQueryMandatoryReviewOutcome::NoEffect(no_effect, mandatory) => Self::NoEffect(
                no_effect.cause(),
                BankEstateMandatoryReview::from_query(mandatory),
            ),
            WorthQueryMandatoryReviewOutcome::Reviewed(value) => {
                Self::Reviewed(BankReviewedEstateElevation::from_query(value))
            }
            WorthQueryMandatoryReviewOutcome::RecoveredCommitted { receipt, mandatory } => {
                Self::RecoveredCommitted {
                    receipt,
                    mandatory: BankEstateMandatoryReview::from_query(mandatory),
                }
            }
            WorthQueryMandatoryReviewOutcome::Stale(stale, mandatory) => Self::Stale {
                stale_fact_count: stale.stale_fact_count(),
                mandatory: BankEstateMandatoryReview::from_query(mandatory),
            },
            WorthQueryMandatoryReviewOutcome::Cancelled(mandatory) => {
                Self::Cancelled(BankEstateMandatoryReview::from_query(mandatory))
            }
            WorthQueryMandatoryReviewOutcome::TimedOut => Self::TimedOut,
            WorthQueryMandatoryReviewOutcome::Denied(denial, mandatory) => Self::Denied {
                kind: denial_kind(denial.kind()),
                stage: denial_stage(denial.stage()),
                mandatory: BankEstateMandatoryReview::from_query(mandatory),
            },
            WorthQueryMandatoryReviewOutcome::Aborted(mandatory) => {
                Self::Aborted(BankEstateMandatoryReview::from_query(mandatory))
            }
            WorthQueryMandatoryReviewOutcome::Deferred(deferred) => Self::Deferred(deferred),
            WorthQueryMandatoryReviewOutcome::Indeterminate => Self::Indeterminate,
        }
    }
}
