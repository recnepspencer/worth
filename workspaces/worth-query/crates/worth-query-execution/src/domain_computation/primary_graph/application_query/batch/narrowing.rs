//! Pre-read narrowing of the existing installed schedule and shared loan.
use super::*;

impl WorthQueryApplicationQueryBatchAdmission {
    /// Narrows installed ceilings and freezes the schedule atomically before
    /// any read or custody claim. Scheduled slots and accounting are preserved.
    /// Later source quotes cannot widen the narrowed custody ceiling.
    pub fn narrow_and_freeze_planned_reads(
        &self,
        limits: WorthQueryApplicationQueryBatchLimits,
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        let BatchPolicy::Installed(plan) = &self.policy else {
            return Err(WorthQueryApplicationQueryBatchResourceDenial::ForeignPlan);
        };
        // Claims and reservations already acquire totals before installed policy.
        let totals = self
            .totals
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut plan = plan
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if plan.frozen
            || plan.started
            || totals.admitted_reads != 0
            || totals.work != 0
            || totals.peak_bytes != 0
            || totals.memory_denial.is_some()
        {
            return Err(WorthQueryApplicationQueryBatchResourceDenial::PlanningFrozen);
        }
        if plan.items > limits.maximum_items() {
            return Err(WorthQueryApplicationQueryBatchResourceDenial::ItemLimit {
                required: plan.items,
                maximum: limits.maximum_items(),
            });
        }
        plan.work = plan.work.min(limits.maximum_read_work());
        plan.bytes = plan.bytes.min(limits.maximum_retained_bytes());
        plan.byte_ceiling = Some(limits.maximum_retained_bytes());
        plan.result = plan.result.min(limits.maximum_result_bytes_per_item());
        plan.frozen = true;
        Ok(())
    }
}
