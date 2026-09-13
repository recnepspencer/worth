use crate::branch::ProductBranchObservation;
use crate::lifecycle::RuntimeWorldInstant;
use crate::publication::{
    NoEffectCause, ReservedCompositePublicationAttempt, RuntimeWorldCancellationBoundary,
    RuntimeWorldCancellationToken, SignalComponentPlanPosture,
};
use crate::recovery::ProductUnpublishedCause;

use super::RuntimeWorldOwnerRoot;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The pre-effect boundary checks, in the one order the attempt must see
    /// them. A denial here has provably moved no owner.
    pub(super) fn admit_owner_execution(
        &self,
        attempt: &ReservedCompositePublicationAttempt,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<(), NoEffectCause> {
        let plan = attempt.plan();
        if cancellation
            .check(RuntimeWorldCancellationBoundary::BeforeFirstOwnerEffect)
            .is_err()
        {
            return Err(NoEffectCause::CancelledBeforeEffect);
        }
        if self.deadline_expired(attempt.deadline()) {
            return Err(NoEffectCause::DeadlineBeforeEffect);
        }
        if !plan.is_internally_consistent() {
            return Err(NoEffectCause::PreEffectFailure);
        }
        if matches!(
            plan.relational().posture(),
            crate::publication::RelationalComponentPlanPosture::RetainExact
        ) && matches!(
            plan.signal().posture(),
            SignalComponentPlanPosture::RetainExact
        ) {
            return Err(NoEffectCause::PreEffectFailure);
        }
        // Owner-local movement does not invalidate a retained product basis.
        // Only the product cell decides whether this expected head is stale;
        // each changing owner admits its own exact mutation at execution.
        if !self.current_product_head_is(attempt.expected_head()) {
            return Err(NoEffectCause::StaleExpectedProductHead);
        }
        self.state
            .bridge
            .compare_current_exact(attempt.predecessor_basis().correspondence_basis())
            .map_err(|_| NoEffectCause::CorrespondenceRebindRequired)?;
        // The exact product cell is rechecked last, immediately before the
        // first owner effect, so a concurrent product publication cannot make
        // this attempt appear current after its earlier checks passed.
        if !self.current_product_head_is(attempt.expected_head()) {
            return Err(NoEffectCause::StaleExpectedProductHead);
        }
        Ok(())
    }

    /// The gate between a settled Relational effect and the Signal advance.
    /// Every denial here stops before the Signal owner is contacted.
    pub(super) fn pre_advance_signal_gate(
        &self,
        expected_head: &ProductBranchObservation,
        deadline: Option<RuntimeWorldInstant>,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<(), (ProductUnpublishedCause, NoEffectCause)> {
        #[cfg(test)]
        super::rehearsal::reach_between_owner_effects(self.owner_identity());
        if cancellation
            .check(RuntimeWorldCancellationBoundary::BetweenOwnerEffects)
            .is_err()
        {
            return Err((
                ProductUnpublishedCause::CancellationAfterEffect,
                NoEffectCause::CancelledBeforeEffect,
            ));
        }
        if self.deadline_expired(deadline) {
            return Err((
                ProductUnpublishedCause::DeadlineAfterEffect,
                NoEffectCause::DeadlineBeforeEffect,
            ));
        }
        if !self.current_product_head_is(expected_head) {
            return Err((
                ProductUnpublishedCause::StaleProductHead,
                NoEffectCause::StaleExpectedProductHead,
            ));
        }
        Ok(())
    }

    pub(crate) fn deadline_expired(&self, deadline: Option<RuntimeWorldInstant>) -> bool {
        deadline.is_some_and(|deadline| self.state.clock.now() >= deadline)
    }
}
