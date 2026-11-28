use worth_signal::facade::{IntervalWakeRegeneration, TemporalRetiredWakeCompactionReport};

use super::*;
use crate::conditional_execution::execution_view::BridgeConditionalExecutionView;

impl BridgeConditionalExecutionView<'_> {
    pub fn regenerate_managed_interval_wake(
        &self,
        binding: &BridgeManagedClockBinding,
        wake: &BridgeManagedDueWake,
    ) -> Result<IntervalWakeRegeneration, BridgeManagedTemporalDenial> {
        let cell = self.admit_managed_clock_lane(binding)?;
        let mut lane = lock_lane(&cell)?;
        lifecycle::require_clock_lane(&lane, binding)?;

        if wake.binding_identity != binding.binding_identity || !lane.retains_own_due_wake(wake) {
            return Err(BridgeManagedTemporalDenial::new(
                BridgeManagedTemporalDenialKind::InvalidContract,
                "interval regeneration requires the current due wake of the supplied clock",
            ));
        }

        lane.begin_effect()?;
        let result = lane.signal.regenerate_interval_wake(wake.signal_wake_id);
        let report = lane.admit_signal_result(result)?;
        lane.after_signal_before_publication()?;
        let successor = report.scheduled();
        let intent = lane
            .intents
            .get_mut(&wake.intent_identity)
            .expect("admitted intent");
        intent.wake_id = successor.id();
        intent.due_coordinate = successor.due_tick().get();
        lane.wake_to_intent.remove(&wake.signal_wake_id);
        lane.wake_to_intent
            .insert(successor.id(), wake.intent_identity.clone());
        lane.finish_effect();
        Ok(report)
    }

    pub fn compact_managed_retired_wakes(
        &self,
        binding: &BridgeManagedClockBinding,
        retained_limit: u32,
        max_expired: u32,
    ) -> Result<TemporalRetiredWakeCompactionReport, BridgeManagedTemporalDenial> {
        let cell = self.admit_managed_clock_lane(binding)?;
        let lane = lock_lane(&cell)?;
        lifecycle::require_clock_lane(&lane, binding)?;
        lane.signal
            .compact_retired_temporal_wake_tail(retained_limit, max_expired)
            .map_err(clock_lane::signal_denial)
    }
}
