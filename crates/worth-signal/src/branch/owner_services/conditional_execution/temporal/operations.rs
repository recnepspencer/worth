use crate::data::temporal::{
    ClockAdvanceRequest, ClockTick, RetiredTemporalWake, ScheduledTemporalWake, TemporalCondition,
    TemporalWakeId, TemporalWakeReschedule, TemporalWakeRetirementReason, TemporalWakeSummary,
    ValidatedClockAdvance,
};

use super::partition::SignalConditionalTemporalState;
use super::{
    SignalConditionalTemporalPartition, SignalConditionalTemporalPartitionDenial as Denial,
};

impl<D, I, T> SignalConditionalTemporalPartition<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn advance_clock(
        &self,
        request: ClockAdvanceRequest,
    ) -> Result<ValidatedClockAdvance, Denial> {
        self.with_admitted_partition(|state| {
            state.admit_clock_ordinal_capacity()?;
            let validated = state
                .temporal
                .validate_clock_advance(request)
                .map_err(Denial::TemporalOperation)?;
            state.temporal.apply_clock_advance(validated);
            Ok(validated)
        })
    }

    pub fn schedule_temporal_wake(
        &self,
        condition: TemporalCondition,
        due_tick: ClockTick,
    ) -> Result<ScheduledTemporalWake, Denial> {
        self.with_admitted_partition(|state| {
            state.admit_active_wake_capacity()?;
            state.admit_wake_identity_capacity(1, 1)?;
            state
                .temporal
                .schedule_wake(condition, due_tick, None)
                .map_err(Denial::TemporalOperation)
        })
    }

    pub fn supersede_temporal_wake(
        &self,
        wake_id: TemporalWakeId,
        condition: TemporalCondition,
        due_tick: ClockTick,
    ) -> Result<TemporalWakeReschedule, Denial> {
        self.with_admitted_partition(|state| {
            state.admit_wake_identity_capacity(1, 2)?;
            let result = state
                .temporal
                .supersede_wake_with_condition(wake_id, condition, due_tick, None)
                .map_err(Denial::TemporalOperation)?;
            state.temporal.forget_retired_wake(wake_id);
            Ok(result)
        })
    }

    pub fn retire_temporal_wake(
        &self,
        wake_id: TemporalWakeId,
        reason: TemporalWakeRetirementReason,
    ) -> Result<RetiredTemporalWake, Denial> {
        self.with_admitted_partition(|state| {
            state.admit_wake_identity_capacity(0, 1)?;
            let retired = state
                .temporal
                .retire_wake(wake_id, reason, None)
                .map_err(Denial::TemporalOperation)?;
            state.temporal.forget_retired_wake(wake_id);
            Ok(retired)
        })
    }

    /// Counts retained active wakes; retired receipts are returned but never retained.
    /// Available for cleanup after quarantine; this read grants no execution authority.
    pub fn temporal_wake_summary(&self) -> Result<TemporalWakeSummary, Denial> {
        self.inspect_retained_wakes()
    }
}

impl SignalConditionalTemporalState {
    pub(super) fn admit_clock_ordinal_capacity(&self) -> Result<(), Denial> {
        self.temporal
            .clock_basis()
            .last_advance_ordinal()
            .get()
            .checked_add(1)
            .ok_or(Denial::IdentityExhausted)?;
        Ok(())
    }

    fn admit_active_wake_capacity(&self) -> Result<(), Denial> {
        let summary = self.temporal.wake_summary();
        if summary
            .scheduled_count()
            .saturating_add(summary.ready_count())
            >= self.maximum_active_wakes as u64
        {
            return Err(Denial::ActiveWakeCapacityExhausted {
                maximum_active_wakes: self.maximum_active_wakes,
            });
        }
        Ok(())
    }

    pub(super) fn admit_wake_identity_capacity(
        &self,
        wake_ids: u64,
        ordinals: u64,
    ) -> Result<(), Denial> {
        let summary = self.temporal.wake_summary();
        summary
            .next_wake_id()
            .get()
            .checked_add(wake_ids)
            .ok_or(Denial::IdentityExhausted)?;
        summary
            .next_wake_ordinal()
            .get()
            .checked_add(ordinals)
            .ok_or(Denial::IdentityExhausted)?;
        Ok(())
    }
}
