use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::EntityId;

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationStaleAttempt,
    WorthQueryElevationClosureKind, WorthQueryMandatoryReview,
};
use crate::domain_computation::authorization::WorthQueryMandatoryReviewBinding;

/// Terminal move-only lifecycle receipt produced by exact mandatory review.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryReviewedElevation;
///
/// fn reviewed_elevation_cannot_be_copied(reviewed: WorthQueryReviewedElevation) {
///     let _copied = reviewed.clone();
/// }
/// ```
#[derive(Debug)]
pub struct WorthQueryReviewedElevation {
    binding: WorthQueryMandatoryReviewBinding,
    review_commit: WorthQueryApplicationCommitReceipt,
}

impl WorthQueryReviewedElevation {
    pub fn publication_source(&self) -> super::WorthQueryApplicationCommitPublicationSource {
        self.review_commit.publication_source()
    }

    pub const fn reviewer(&self) -> EntityId {
        self.binding.reviewer()
    }

    pub const fn reviewed_at(&self) -> &AspectValue {
        self.binding.reviewed_at()
    }

    pub const fn closure_kind(&self) -> WorthQueryElevationClosureKind {
        self.binding.mandatory().closure_kind()
    }

    pub const fn requester(&self) -> EntityId {
        self.binding.mandatory().requester()
    }

    pub const fn approver(&self) -> EntityId {
        self.binding.mandatory().approver()
    }

    pub const fn closer(&self) -> EntityId {
        self.binding.mandatory().closer()
    }

    pub const fn resource(&self) -> EntityId {
        self.binding.mandatory().resource()
    }

    pub const fn grant(&self) -> EntityId {
        self.binding.mandatory().grant()
    }

    pub const fn elevation(&self) -> EntityId {
        self.binding.mandatory().elevation()
    }

    pub const fn review(&self) -> EntityId {
        self.binding.mandatory().review()
    }

    pub const fn action(&self) -> &AspectValue {
        self.binding.mandatory().action()
    }

    pub const fn purpose(&self) -> &AspectValue {
        self.binding.mandatory().purpose()
    }

    pub const fn field(&self) -> Option<&AspectValue> {
        self.binding.mandatory().field()
    }

    pub const fn magnitude(&self) -> Option<&AspectValue> {
        self.binding.mandatory().magnitude()
    }

    pub const fn cardinality(&self) -> u32 {
        self.binding.mandatory().cardinality()
    }

    pub const fn reason(&self) -> &AspectValue {
        self.binding.mandatory().reason()
    }

    pub const fn issued_at(&self) -> &AspectValue {
        self.binding.mandatory().issued_at()
    }

    pub const fn expires_at(&self) -> &AspectValue {
        self.binding.mandatory().expires_at()
    }
}

#[derive(Debug)]
pub enum WorthQueryMandatoryReviewOutcome {
    ProductStale(
        crate::domain_computation::WorthQueryProductStaleApplication,
        WorthQueryMandatoryReview,
    ),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    Reviewed(WorthQueryReviewedElevation),
    AlreadyReviewed(WorthQueryReviewedElevation),
    Stale(WorthQueryApplicationStaleAttempt, WorthQueryMandatoryReview),
    Cancelled(WorthQueryMandatoryReview),
    TimedOut,
    Denied(WorthQueryApplicationCommitDenial, WorthQueryMandatoryReview),
    Aborted(WorthQueryMandatoryReview),
    Deferred(super::WorthQueryApplicationCommitDeferred),
    SettlementDeferred(super::WorthQueryApplicationSettlementDeferred),
    Indeterminate,
}

pub(in crate::domain_computation::primary_graph) fn reviewed_outcome(
    outcome: WorthQueryApplicationCommitOutcome,
    binding: WorthQueryMandatoryReviewBinding,
) -> WorthQueryMandatoryReviewOutcome {
    match outcome {
        WorthQueryApplicationCommitOutcome::Committed(commit) => {
            WorthQueryMandatoryReviewOutcome::Reviewed(reviewed(binding, commit))
        }
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(commit) => {
            WorthQueryMandatoryReviewOutcome::AlreadyReviewed(reviewed(binding, commit))
        }
        WorthQueryApplicationCommitOutcome::Stale(stale) => {
            WorthQueryMandatoryReviewOutcome::Stale(stale, binding.into_mandatory())
        }
        WorthQueryApplicationCommitOutcome::ProductStale(stale) => {
            WorthQueryMandatoryReviewOutcome::ProductStale(stale, binding.into_mandatory())
        }
        WorthQueryApplicationCommitOutcome::Cancelled => {
            WorthQueryMandatoryReviewOutcome::Cancelled(binding.into_mandatory())
        }
        WorthQueryApplicationCommitOutcome::TimedOut => WorthQueryMandatoryReviewOutcome::TimedOut,
        WorthQueryApplicationCommitOutcome::Denied(denial) => {
            WorthQueryMandatoryReviewOutcome::Denied(denial, binding.into_mandatory())
        }
        WorthQueryApplicationCommitOutcome::Aborted => {
            WorthQueryMandatoryReviewOutcome::Aborted(binding.into_mandatory())
        }
        WorthQueryApplicationCommitOutcome::Deferred(deferred) => {
            WorthQueryMandatoryReviewOutcome::Deferred(deferred)
        }
        WorthQueryApplicationCommitOutcome::ProductUnpublished(unpublished) => {
            WorthQueryMandatoryReviewOutcome::ProductUnpublished(unpublished)
        }
        WorthQueryApplicationCommitOutcome::SettlementDeferred(deferred) => {
            WorthQueryMandatoryReviewOutcome::SettlementDeferred(deferred)
        }
        WorthQueryApplicationCommitOutcome::Indeterminate(_) => {
            WorthQueryMandatoryReviewOutcome::Indeterminate
        }
    }
}

fn reviewed(
    binding: WorthQueryMandatoryReviewBinding,
    review_commit: WorthQueryApplicationCommitReceipt,
) -> WorthQueryReviewedElevation {
    WorthQueryReviewedElevation {
        binding,
        review_commit,
    }
}
