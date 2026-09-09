use super::{
    BridgeManagedClockLane, BridgeManagedDueWake, BridgeManagedDueWakeBatch,
    BridgeManagedTemporalDenial,
};
use std::sync::Arc;

impl BridgeManagedClockLane {
    pub(super) fn promote_due(
        &mut self,
        binding_identity: &Arc<str>,
    ) -> Result<BridgeManagedDueWakeBatch, BridgeManagedTemporalDenial> {
        let capacity = self
            .maximum_due_wakes_per_observation
            .get()
            .min(self.maximum_active_intents);
        let reservation = super::retention::reserve_due_output(&self.retention, capacity)?;
        self.begin_effect()?;
        let bounded = self
            .signal
            .promote_due_temporal_wakes_ready_bounded(self.maximum_due_wakes_per_observation);
        let bounded = self.admit_signal_result(bounded)?;
        self.after_signal_before_publication()?;
        let due = self.join_promoted_due(binding_identity, bounded, &reservation)?;
        self.finish_effect();
        Ok(due)
    }

    pub(super) fn advance_and_promote_due(
        &mut self,
        binding_identity: &Arc<str>,
        coordinate: u64,
    ) -> Result<(u64, BridgeManagedDueWakeBatch), BridgeManagedTemporalDenial> {
        let capacity = self
            .maximum_due_wakes_per_observation
            .get()
            .min(self.maximum_active_intents);
        let reservation = super::retention::reserve_due_output(&self.retention, capacity)?;
        self.begin_effect()?;
        let result = self
            .signal
            .advance_clock_and_promote_due_temporal_wakes_ready_bounded(
                worth_signal::facade::ClockAdvanceRequest::new(
                    worth_signal::facade::ClockDomain::MonotonicExecution,
                    worth_signal::facade::ClockTick::new(coordinate),
                ),
                self.maximum_due_wakes_per_observation,
            );
        let (advance, bounded) = self.admit_signal_result(result)?;
        self.after_signal_before_publication()?;
        let due = self.join_promoted_due(binding_identity, bounded, &reservation)?;
        self.finish_effect();
        Ok((advance.ordinal().get(), due))
    }

    fn join_promoted_due(
        &self,
        binding_identity: &Arc<str>,
        bounded: worth_signal::facade::branch::SignalConditionalTemporalPromotion,
        reservation: &Arc<super::super::retention::BridgeRetentionReservation>,
    ) -> Result<BridgeManagedDueWakeBatch, BridgeManagedTemporalDenial> {
        let promotion = bounded.promotion();
        let frontier_before = promotion.frontier_before();
        let frontier_after = promotion.frontier_after();
        let mut wakes = Vec::with_capacity(promotion.ready_wakes().len());
        for wake in bounded.promotion().ready_wakes() {
            wakes.push(self.join_due_wake(binding_identity, wake, reservation)?);
        }
        Ok(BridgeManagedDueWakeBatch {
            wakes: super::BridgeManagedDueWakeBuffer {
                wakes,
                reservation: Arc::clone(reservation),
            },
            due_work_remaining: bounded.due_work_remaining(),
            frontier_width_before: frontier_before.scheduled_frontier_width(),
            frontier_width_after: frontier_after.scheduled_frontier_width(),
        })
    }

    fn join_due_wake(
        &self,
        binding_identity: &Arc<str>,
        wake: &worth_signal::facade::ReadyTemporalWake,
        reservation: &Arc<super::super::retention::BridgeRetentionReservation>,
    ) -> Result<BridgeManagedDueWake, BridgeManagedTemporalDenial> {
        let identity = self
            .wake_to_intent
            .get(&wake.id())
            .ok_or_else(super::quarantine::denied)?;
        let intent = self
            .intents
            .get(identity)
            .filter(|intent| intent.wake_id == wake.id())
            .ok_or_else(super::quarantine::denied)?;
        Ok(BridgeManagedDueWake {
            _reservation: Arc::clone(reservation),
            binding_identity: Arc::clone(binding_identity),
            intent_identity: identity.clone(),
            revision: intent.revision,
            idempotency_identity: Arc::clone(&intent.idempotency_identity),
            source_record_identity: intent.source_record_identity,
            due_coordinate: intent.due_coordinate,
            ready_coordinate: wake.ready_tick().get(),
            signal_wake_id: wake.id(),
            scheduled_ordinal: wake.scheduled_ordinal(),
            ready_ordinal: wake.ready_ordinal(),
            observation_baselines: Arc::clone(&intent.observation_baselines),
        })
    }
}
