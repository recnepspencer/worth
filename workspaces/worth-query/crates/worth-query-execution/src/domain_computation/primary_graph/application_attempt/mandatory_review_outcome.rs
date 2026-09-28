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

/// Every way committing a mandatory review can end.
///
/// This mirrors the application commit outcome. `Reviewed` and
/// `AlreadyReviewed` carry the move-only reviewed-elevation receipt. Most
/// variants that did not land hand the consumed mandatory review back, so it
/// can be committed again.
#[derive(Debug)]
pub enum WorthQueryMandatoryReviewOutcome {
    /// The product branch moved after the basis. Nothing was committed.
    ProductStale(
        crate::domain_computation::WorthQueryProductStaleApplication,
        WorthQueryMandatoryReview,
    ),
    /// Some owners moved, but the product head did not. Recover publication.
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    /// Nothing was written; the cause says why.
    NoEffect(
        super::WorthQueryApplicationNoEffect,
        WorthQueryMandatoryReview,
    ),
    /// This attempt committed the mandatory review.
    Reviewed(WorthQueryReviewedElevation),
    /// The idempotency key had already committed this mandatory review; nothing was redone.
    AlreadyReviewed(WorthQueryReviewedElevation),
    /// The basis was stale at compare. Nothing was committed; re-read and retry.
    Stale(WorthQueryApplicationStaleAttempt, WorthQueryMandatoryReview),
    /// The attempt was cancelled before it landed.
    Cancelled(WorthQueryMandatoryReview),
    /// The attempt reached its deadline before it landed.
    TimedOut,
    /// The commit was refused before publication.
    Denied(WorthQueryApplicationCommitDenial, WorthQueryMandatoryReview),
    /// The commit's answer was lost, and a re-read found it never landed.
    Aborted(WorthQueryMandatoryReview),
    /// A capacity or lifetime limit stopped the attempt. Retry later.
    Deferred(super::WorthQueryApplicationCommitDeferred),
    /// The branch moved, but settlement did not finish. Recover the settlement.
    SettlementDeferred(super::WorthQueryApplicationSettlementDeferred),
    /// Whether the commit landed is unresolved. Do not retry blindly.
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
            match commit
                .committed_changes()
                .committed_field_values(binding.review(), &[binding.reviewed_at_field()])
            {
                Some(values) => WorthQueryMandatoryReviewOutcome::AlreadyReviewed(reviewed(
                    binding.restore_committed_review(values[0].clone()),
                    commit,
                )),
                None => WorthQueryMandatoryReviewOutcome::Denied(
                    WorthQueryApplicationCommitDenial::provider_rejected_with_detail(
                        super::WorthQueryApplicationCommitDenialStage::Idempotency,
                        "committed review field is unavailable",
                    ),
                    binding.into_mandatory(),
                ),
            }
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
        WorthQueryApplicationCommitOutcome::NoEffect(no_effect) => {
            WorthQueryMandatoryReviewOutcome::NoEffect(no_effect, binding.into_mandatory())
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
