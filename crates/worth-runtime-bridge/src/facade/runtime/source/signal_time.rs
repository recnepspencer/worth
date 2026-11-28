use std::num::NonZeroUsize;

use worth_signal::facade::runtime::BoundedTemporalReadyPromotionSummary;
use worth_signal::facade::{
    ClockAdvanceRequest, IntervalWakeRegeneration, SignalError,
    TemporalRetiredWakeCompactionReport, TemporalWakeId,
};

use crate::facade::{BridgeTemporalCondition, RuntimeBridge};

/// An issued wake on the original Bridge Signal runtime.
pub struct BridgeSignalWake {
    runtime_key: u64,
    id: TemporalWakeId,
}

impl BridgeSignalWake {
    pub fn id(&self) -> TemporalWakeId {
        self.id
    }

    pub(super) fn require_owner(&self, bridge: &RuntimeBridge) -> Result<(), SignalError> {
        if self.runtime_key == bridge.signal_runtime_key {
            Ok(())
        } else {
            Err(SignalError::invalid_input(
                "temporal wake belongs to a different Bridge runtime",
            ))
        }
    }
}

impl RuntimeBridge {
    pub fn schedule_signal_wake(
        &self,
        condition: BridgeTemporalCondition,
    ) -> Result<BridgeSignalWake, SignalError> {
        self.with_signal_source(|runtime| {
            let condition = condition.into_condition();
            let due = condition.first_due_tick(runtime.clock_basis().current_tick());
            let scheduled = runtime.schedule_temporal_wake(condition, due)?;
            Ok(BridgeSignalWake {
                runtime_key: self.signal_runtime_key,
                id: scheduled.id(),
            })
        })
    }

    pub fn advance_signal_clock(
        &self,
        advance: ClockAdvanceRequest,
        maximum_ready: NonZeroUsize,
    ) -> Result<BoundedTemporalReadyPromotionSummary, SignalError> {
        self.with_signal_source(|runtime| {
            runtime.advance_clock(advance)?;
            runtime.promote_due_temporal_wakes_ready_bounded(maximum_ready)
        })
    }

    pub fn regenerate_signal_interval_wake(
        &self,
        wake: &BridgeSignalWake,
    ) -> Result<(BridgeSignalWake, IntervalWakeRegeneration), SignalError> {
        wake.require_owner(self)?;
        self.with_signal_source(|runtime| {
            let report = runtime.regenerate_interval_wake(wake.id)?;
            Ok((
                BridgeSignalWake {
                    runtime_key: self.signal_runtime_key,
                    id: report.scheduled().id(),
                },
                report,
            ))
        })
    }

    pub fn compact_retired_signal_wakes(
        &self,
        retained_limit: u32,
        max_expired: u32,
    ) -> Result<TemporalRetiredWakeCompactionReport, SignalError> {
        self.with_signal_source(|runtime| {
            Ok(runtime.compact_retired_temporal_wake_tail(retained_limit, max_expired))
        })
    }
}
