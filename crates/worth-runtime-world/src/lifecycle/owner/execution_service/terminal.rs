use crate::branch::ProductBranchReferenceSnapshot;
use crate::publication::{
    CompositeAttemptProgress, NoEffectCause, NoEffectCompositePublication, OwnerExecutionOutcome,
    RelationalAttemptProgress, ReservedCompositePublicationAttempt,
    RuntimeWorldCancellationBoundary, RuntimeWorldCancellationToken,
};
use crate::recovery::ProductUnpublishedCause;

use super::RuntimeWorldOwnerRoot;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The post-effect boundary checks and the settled terminal. Every denial
    /// reached from here retains the owner effects already performed; none of
    /// them can be reclassified as no-effect.
    pub(super) fn publish_settled_progress(
        &self,
        mut attempt: ReservedCompositePublicationAttempt,
        progress: CompositeAttemptProgress,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> OwnerExecutionOutcome {
        if cancellation
            .check(RuntimeWorldCancellationBoundary::BeforeProductMovement)
            .is_err()
        {
            attempt.observe_cancellation();
            return self.retain_after_effect(
                attempt,
                progress,
                ProductUnpublishedCause::CancellationAfterEffect,
                None,
            );
        }
        if self.deadline_expired(attempt.deadline()) {
            return self.retain_after_effect(
                attempt,
                progress,
                ProductUnpublishedCause::DeadlineAfterEffect,
                None,
            );
        }
        if !self.current_product_head_is(attempt.expected_head()) {
            return self.retain_or_no_effect(
                attempt,
                progress,
                ProductUnpublishedCause::StaleProductHead,
                NoEffectCause::StaleExpectedProductHead,
            );
        }
        let successor =
            self.issue_successor_basis_from_progress(&progress, attempt.predecessor_basis());
        if !self.successor_correspondence_is_valid(&successor) {
            return self.retain_or_no_effect(
                attempt,
                progress,
                ProductUnpublishedCause::CorrespondenceRebindRequired,
                NoEffectCause::CorrespondenceRebindRequired,
            );
        }
        OwnerExecutionOutcome::Settled(attempt.settle_with_successor_basis(progress, successor))
    }

    pub(super) fn retain_after_effect(
        &self,
        attempt: ReservedCompositePublicationAttempt,
        progress: CompositeAttemptProgress,
        cause: ProductUnpublishedCause,
        observed: Option<ProductBranchReferenceSnapshot>,
    ) -> OwnerExecutionOutcome {
        let successor =
            self.issue_successor_basis_from_progress(&progress, attempt.predecessor_basis());
        OwnerExecutionOutcome::ProductUnpublished(
            attempt
                .settle(progress)
                .retain_with_cause(successor, cause, observed),
        )
    }

    pub(super) fn retain_or_no_effect(
        &self,
        attempt: ReservedCompositePublicationAttempt,
        progress: CompositeAttemptProgress,
        cause: ProductUnpublishedCause,
        no_effect: NoEffectCause,
    ) -> OwnerExecutionOutcome {
        if progress.owner_effect_count() == 0 {
            return self.no_effect(attempt, no_effect);
        }
        let observed = self.current_product_head_snapshot(attempt.expected_head());
        self.retain_after_effect(attempt, progress, cause, observed)
    }

    pub(super) fn no_effect(
        &self,
        attempt: ReservedCompositePublicationAttempt,
        cause: NoEffectCause,
    ) -> OwnerExecutionOutcome {
        assert_eq!(
            attempt.progress().owner_effect_count(),
            0,
            "performed owner evidence cannot become no-effect"
        );
        let expected = attempt.expected_head().clone();
        let observed = (cause == NoEffectCause::StaleExpectedProductHead)
            .then(|| self.current_product_head_snapshot(&expected))
            .flatten();
        OwnerExecutionOutcome::NoEffect(
            NoEffectCompositePublication::new(cause, Some(expected)).with_observed_head(observed),
        )
    }
}

impl CompositeAttemptProgress {
    pub(super) fn into_relational(self) -> RelationalAttemptProgress {
        let (relational, _) = self.into_parts();
        relational
    }
}
