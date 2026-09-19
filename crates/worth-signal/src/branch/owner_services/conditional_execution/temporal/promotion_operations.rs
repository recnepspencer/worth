use std::num::NonZeroUsize;

use crate::data::retained_storage::{RetainedStorageCharge, SignalConditionalRetentionReservation};
use crate::data::temporal::{
    BoundedTemporalReadyPromotionSummary, ClockAdvanceRequest, ReadyTemporalWake,
    TemporalReadyPromotionSummary, ValidatedClockAdvance,
};

use super::partition::SignalConditionalTemporalState;
use super::retention::map_retention_denial;
use super::{
    SignalConditionalTemporalPartition, SignalConditionalTemporalPartitionDenial as Denial,
    SignalConditionalTemporalPromotion,
};

impl<D, I, T> SignalConditionalTemporalPartition<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Visits only the indexed due prefix, capped by the caller's selection bound.
    /// Output custody is admitted before changing a wake ordinal or frontier.
    pub fn promote_due_temporal_wakes_ready_bounded(
        &self,
        maximum: NonZeroUsize,
    ) -> Result<SignalConditionalTemporalPromotion, Denial> {
        self.with_admitted_partition(|state| {
            let prepared = state.prepare_promotion(maximum)?;
            state.commit_promotion(prepared, None)
        })
    }

    /// Reserves the returned batch before applying either clock or wake effects.
    pub fn advance_clock_and_promote_due_temporal_wakes_ready_bounded(
        &self,
        request: ClockAdvanceRequest,
        maximum: NonZeroUsize,
    ) -> Result<(ValidatedClockAdvance, SignalConditionalTemporalPromotion), Denial> {
        self.with_admitted_partition(|state| {
            state.admit_clock_ordinal_capacity()?;
            let validated = state
                .temporal
                .validate_clock_advance(request)
                .map_err(Denial::TemporalOperation)?;
            let prepared = state.prepare_promotion(maximum)?;
            Ok((
                validated,
                state.commit_promotion(prepared, Some(validated))?,
            ))
        })
    }
}

struct SignalConditionalTemporalPromotionPreparation {
    ready: Vec<ReadyTemporalWake>,
    custody: SignalConditionalRetentionReservation,
    maximum: NonZeroUsize,
}

impl SignalConditionalTemporalState {
    /// Preflight has already admitted the entire output and ordinal capacity.
    /// Fence before any effect and clear only after complete result construction.
    /// An impossible divergence or unwind therefore cannot reopen partial state
    /// as ordinary retryable work. Quarantine does not release retained custody.
    fn commit_promotion(
        &mut self,
        prepared: SignalConditionalTemporalPromotionPreparation,
        advance: Option<ValidatedClockAdvance>,
    ) -> Result<SignalConditionalTemporalPromotion, Denial> {
        self.quarantined = true;
        let result = self.apply_clock_and_promotion(prepared, advance);
        match result {
            Ok(result) => {
                self.quarantined = false;
                Ok(result)
            }
            Err(_) => Err(Denial::PartitionQuarantined),
        }
    }

    fn apply_clock_and_promotion(
        &mut self,
        prepared: SignalConditionalTemporalPromotionPreparation,
        advance: Option<ValidatedClockAdvance>,
    ) -> Result<SignalConditionalTemporalPromotion, Denial> {
        if let Some(advance) = advance {
            self.temporal.apply_clock_advance(advance);
            #[cfg(test)]
            self.reach_promotion_fault_boundary(
                super::promotion_fault::SignalTemporalPromotionBoundary::AfterClockAdvance,
            )?;
        }
        self.apply_promotion(prepared)
    }

    fn prepare_promotion(
        &self,
        maximum: NonZeroUsize,
    ) -> Result<SignalConditionalTemporalPromotionPreparation, Denial> {
        let scheduled = usize::try_from(self.temporal.wake_summary().scheduled_count())
            .map_err(|_| Denial::StorageChargeOverflow)?;
        let count = scheduled.min(maximum.get());
        self.admit_wake_identity_capacity(0, count as u64)?;
        let charge = RetainedStorageCharge::capacity::<ReadyTemporalWake>(count)
            .and_then(|buffer| {
                buffer.checked_add(RetainedStorageCharge::capacity::<
                    BoundedTemporalReadyPromotionSummary,
                >(1)?)
            })
            .map_err(|_| Denial::StorageChargeOverflow)?;
        let custody = self
            .custody
            .ledger()
            .reserve_temporal(0, 0, charge)
            .map_err(map_retention_denial)?;
        // Rust 1.94's Vec::with_capacity allocates the requested layout and
        // retains precisely this capacity. No push below exceeds that bound.
        // Recheck alongside the pinned B-tree representation on toolchain changes.
        let ready = Vec::with_capacity(count);
        debug_assert_eq!(ready.capacity(), count);
        Ok(SignalConditionalTemporalPromotionPreparation {
            ready,
            custody,
            maximum,
        })
    }

    fn apply_promotion(
        &mut self,
        mut prepared: SignalConditionalTemporalPromotionPreparation,
    ) -> Result<SignalConditionalTemporalPromotion, Denial> {
        let frontier_before = self.temporal.frontier_snapshot();
        let current_tick = self.temporal.clock_basis().current_tick();
        while prepared.ready.len() < prepared.maximum.get() {
            let frontier = self.temporal.frontier_snapshot();
            if !frontier
                .next_due_tick()
                .is_some_and(|tick| tick <= current_tick)
            {
                break;
            }
            let wake_id = frontier
                .next_due_wake_id()
                .ok_or(Denial::PartitionQuarantined)?;
            prepared.ready.push(
                self.temporal
                    .promote_wake_ready(wake_id, None)
                    .map_err(Denial::TemporalOperation)?,
            );
            #[cfg(test)]
            if prepared.ready.len() == 1 {
                self.reach_promotion_fault_boundary(
                    super::promotion_fault::SignalTemporalPromotionBoundary::AfterFirstPromotion,
                )?;
            }
        }
        let frontier_after = self.temporal.frontier_snapshot();
        let remaining = frontier_after
            .next_due_tick()
            .is_some_and(|tick| tick <= current_tick);
        Ok(SignalConditionalTemporalPromotion {
            summary: BoundedTemporalReadyPromotionSummary::new(
                TemporalReadyPromotionSummary::new(
                    frontier_before,
                    frontier_after,
                    prepared.ready,
                    1,
                ),
                prepared.maximum.get(),
                remaining,
            ),
            _custody: prepared.custody,
        })
    }
}
