//! Bank-owned classifications for every estate elevation transition.

use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDeferred, WorthQueryApplicationSettlementDeferred,
    WorthQueryElevationApprovalOutcome, WorthQueryElevationCloseOutcome,
    WorthQueryElevationRequestOutcome, WorthQueryMandatoryReviewOutcome,
    WorthQueryProductStaleApplication, WorthQueryProductUnpublishedApplication,
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
    ProductStale(WorthQueryProductStaleApplication),
    ProductUnpublished(WorthQueryProductUnpublishedApplication),
    NoEffect(WorthQueryApplicationNoEffectCause),
    Requested(BankRequestedEstateElevation),
    AlreadyRequested(BankRequestedEstateElevation),
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
    SettlementDeferred(WorthQueryApplicationSettlementDeferred),
    Indeterminate,
}

#[derive(Debug)]
pub enum BankEstateElevationApprovalOutcome {
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
    AlreadyApproved(BankApprovedEstateElevation),
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
    SettlementDeferred(WorthQueryApplicationSettlementDeferred),
    Indeterminate,
}

#[derive(Debug)]
pub enum BankEstateElevationCloseOutcome {
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
    AlreadyClosed(BankEstateMandatoryReview),
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
    SettlementDeferred(WorthQueryApplicationSettlementDeferred),
    Indeterminate,
}

#[derive(Debug)]
pub enum BankEstateMandatoryReviewOutcome {
    ProductStale(WorthQueryProductStaleApplication, BankEstateMandatoryReview),
    ProductUnpublished(WorthQueryProductUnpublishedApplication),
    NoEffect(
        WorthQueryApplicationNoEffectCause,
        BankEstateMandatoryReview,
    ),
    Reviewed(BankReviewedEstateElevation),
    AlreadyReviewed(BankReviewedEstateElevation),
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
    SettlementDeferred(WorthQueryApplicationSettlementDeferred),
    Indeterminate,
}

impl BankEstateElevationRequestOutcome {
    pub(crate) fn from_query(outcome: WorthQueryElevationRequestOutcome) -> Self {
        match outcome {
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
            WorthQueryElevationRequestOutcome::AlreadyRequested(value) => {
                Self::AlreadyRequested(BankRequestedEstateElevation::from_query(value))
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
            WorthQueryElevationRequestOutcome::SettlementDeferred(deferred) => {
                Self::SettlementDeferred(deferred)
            }
            WorthQueryElevationRequestOutcome::Indeterminate => Self::Indeterminate,
        }
    }
}

impl BankEstateElevationApprovalOutcome {
    pub(crate) fn from_query(outcome: WorthQueryElevationApprovalOutcome) -> Self {
        match outcome {
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
            WorthQueryElevationApprovalOutcome::AlreadyApproved(value) => {
                Self::AlreadyApproved(BankApprovedEstateElevation::from_query(value))
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
            WorthQueryElevationApprovalOutcome::SettlementDeferred(deferred) => {
                Self::SettlementDeferred(deferred)
            }
            WorthQueryElevationApprovalOutcome::Indeterminate => Self::Indeterminate,
        }
    }
}

impl BankEstateElevationCloseOutcome {
    pub(crate) fn from_query(outcome: WorthQueryElevationCloseOutcome) -> Self {
        match outcome {
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
            WorthQueryElevationCloseOutcome::AlreadyClosed(value) => {
                Self::AlreadyClosed(BankEstateMandatoryReview::from_query(value))
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
            WorthQueryElevationCloseOutcome::SettlementDeferred(deferred) => {
                Self::SettlementDeferred(deferred)
            }
            WorthQueryElevationCloseOutcome::Indeterminate => Self::Indeterminate,
        }
    }
}

impl BankEstateMandatoryReviewOutcome {
    pub(crate) fn from_query(outcome: WorthQueryMandatoryReviewOutcome) -> Self {
        match outcome {
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
            WorthQueryMandatoryReviewOutcome::AlreadyReviewed(value) => {
                Self::AlreadyReviewed(BankReviewedEstateElevation::from_query(value))
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
            WorthQueryMandatoryReviewOutcome::SettlementDeferred(deferred) => {
                Self::SettlementDeferred(deferred)
            }
            WorthQueryMandatoryReviewOutcome::Indeterminate => Self::Indeterminate,
        }
    }
}
