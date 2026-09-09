use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::EntityId;

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationStaleAttempt,
    WorthQueryApprovedElevation,
};
use crate::domain_computation::authorization::WorthQueryElevationCloseBinding;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryElevationClosureKind {
    Revoked,
    Expired,
}

/// Move-only obligation produced only after the approved elevation is closed.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryMandatoryReview;
///
/// fn mandatory_review_cannot_be_copied(review: WorthQueryMandatoryReview) {
///     let _copied = review.clone();
/// }
/// ```
#[derive(Debug)]
pub struct WorthQueryMandatoryReview {
    binding: WorthQueryElevationCloseBinding,
    close_commit: WorthQueryApplicationCommitReceipt,
}

impl WorthQueryMandatoryReview {
    pub const fn closure_kind(&self) -> WorthQueryElevationClosureKind {
        self.binding.closure_kind()
    }

    pub const fn closed_at(&self) -> &AspectValue {
        self.binding.closed_at()
    }

    pub const fn closer(&self) -> EntityId {
        self.binding.closer()
    }

    pub fn publication_source(&self) -> super::WorthQueryApplicationCommitPublicationSource {
        self.close_commit.publication_source()
    }

    pub const fn requester(&self) -> EntityId {
        self.binding.approved().requester()
    }

    pub const fn approver(&self) -> EntityId {
        self.binding.approved().approver()
    }

    pub const fn resource(&self) -> EntityId {
        self.binding.approved().resource()
    }

    pub const fn grant(&self) -> EntityId {
        self.binding.approved().grant()
    }

    pub const fn elevation(&self) -> EntityId {
        self.binding.approved().elevation()
    }

    pub const fn review(&self) -> EntityId {
        self.binding.approved().review()
    }

    pub const fn action(&self) -> &AspectValue {
        self.binding.approved().action()
    }

    pub const fn purpose(&self) -> &AspectValue {
        self.binding.approved().purpose()
    }

    pub const fn field(&self) -> Option<&AspectValue> {
        self.binding.approved().field()
    }

    pub const fn magnitude(&self) -> Option<&AspectValue> {
        self.binding.approved().magnitude()
    }

    pub const fn cardinality(&self) -> u32 {
        self.binding.approved().cardinality()
    }

    pub const fn reason(&self) -> &AspectValue {
        self.binding.approved().reason()
    }

    pub const fn issued_at(&self) -> &AspectValue {
        self.binding.approved().issued_at()
    }

    pub const fn expires_at(&self) -> &AspectValue {
        self.binding.approved().expires_at()
    }

    pub(in crate::domain_computation) const fn approved(&self) -> &WorthQueryApprovedElevation {
        self.binding.approved()
    }

    pub(in crate::domain_computation) fn belongs_to_lifecycle(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        branch: &worth_relational::facade::history::BranchId,
        capability_identity: [u8; 32],
        capability_authority_identity: &str,
    ) -> bool {
        self.binding.approved().belongs_to_lifecycle(
            runtime_authority,
            branch,
            capability_identity,
            capability_authority_identity,
        ) && self.close_commit.terminal().branch() == branch
            && self.close_commit.provider_runtime_instance_id()
                == self
                    .binding
                    .approved()
                    .approval_commit_receipt()
                    .provider_runtime_instance_id()
    }
}

#[derive(Debug)]
pub enum WorthQueryElevationCloseOutcome {
    ProductStale(
        crate::domain_computation::WorthQueryProductStaleApplication,
        WorthQueryApprovedElevation,
    ),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    Closed(WorthQueryMandatoryReview),
    AlreadyClosed(WorthQueryMandatoryReview),
    Stale(
        WorthQueryApplicationStaleAttempt,
        WorthQueryApprovedElevation,
    ),
    Cancelled(WorthQueryApprovedElevation),
    TimedOut,
    Denied(
        WorthQueryApplicationCommitDenial,
        WorthQueryApprovedElevation,
    ),
    Aborted(WorthQueryApprovedElevation),
    Deferred(super::WorthQueryApplicationCommitDeferred),
    SettlementDeferred(super::WorthQueryApplicationSettlementDeferred),
    Indeterminate,
}

pub(in crate::domain_computation::primary_graph) fn closed_outcome(
    outcome: WorthQueryApplicationCommitOutcome,
    binding: WorthQueryElevationCloseBinding,
) -> WorthQueryElevationCloseOutcome {
    match outcome {
        WorthQueryApplicationCommitOutcome::Committed(commit) => {
            WorthQueryElevationCloseOutcome::Closed(closed(binding, commit))
        }
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(commit) => {
            WorthQueryElevationCloseOutcome::AlreadyClosed(closed(binding, commit))
        }
        WorthQueryApplicationCommitOutcome::Stale(stale) => {
            WorthQueryElevationCloseOutcome::Stale(stale, binding.into_approved())
        }
        WorthQueryApplicationCommitOutcome::ProductStale(stale) => {
            WorthQueryElevationCloseOutcome::ProductStale(stale, binding.into_approved())
        }
        WorthQueryApplicationCommitOutcome::Cancelled => {
            WorthQueryElevationCloseOutcome::Cancelled(binding.into_approved())
        }
        WorthQueryApplicationCommitOutcome::TimedOut => WorthQueryElevationCloseOutcome::TimedOut,
        WorthQueryApplicationCommitOutcome::Denied(denial) => {
            WorthQueryElevationCloseOutcome::Denied(denial, binding.into_approved())
        }
        WorthQueryApplicationCommitOutcome::Aborted => {
            WorthQueryElevationCloseOutcome::Aborted(binding.into_approved())
        }
        WorthQueryApplicationCommitOutcome::Deferred(deferred) => {
            WorthQueryElevationCloseOutcome::Deferred(deferred)
        }
        WorthQueryApplicationCommitOutcome::ProductUnpublished(unpublished) => {
            WorthQueryElevationCloseOutcome::ProductUnpublished(unpublished)
        }
        WorthQueryApplicationCommitOutcome::SettlementDeferred(deferred) => {
            WorthQueryElevationCloseOutcome::SettlementDeferred(deferred)
        }
        WorthQueryApplicationCommitOutcome::Indeterminate(_) => {
            WorthQueryElevationCloseOutcome::Indeterminate
        }
    }
}

fn closed(
    binding: WorthQueryElevationCloseBinding,
    close_commit: WorthQueryApplicationCommitReceipt,
) -> WorthQueryMandatoryReview {
    WorthQueryMandatoryReview {
        binding,
        close_commit,
    }
}
