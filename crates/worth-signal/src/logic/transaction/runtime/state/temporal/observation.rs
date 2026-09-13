use crate::data::temporal::{
    ReadyTemporalWake, RetiredTemporalWake, ScheduledTemporalWake, TemporalFrontierSnapshot,
    TemporalWakeSummary,
};

use super::super::runtime_state::SignalRuntime;
use super::TemporalRuntimeState;

impl TemporalRuntimeState {
    pub(in crate::logic::transaction::runtime) fn scheduled_wake_evidence(
        &self,
    ) -> Vec<ScheduledTemporalWake> {
        self.scheduled_wakes.values().cloned().collect()
    }

    pub(in crate::logic::transaction::runtime) fn ready_wake_evidence(
        &self,
    ) -> Vec<ReadyTemporalWake> {
        self.ready_wakes.values().cloned().collect()
    }

    pub(in crate::logic::transaction::runtime) fn retired_wake_evidence(
        &self,
    ) -> Vec<RetiredTemporalWake> {
        self.retired_wakes.values().cloned().collect()
    }

    pub fn wake_summary(&self) -> TemporalWakeSummary {
        TemporalWakeSummary::new(
            self.scheduled_wakes.len(),
            self.ready_wakes.len(),
            self.retired_wakes.len(),
            self.next_wake_id,
            self.next_wake_ordinal,
        )
    }

    pub fn frontier_snapshot(&self) -> TemporalFrontierSnapshot {
        let next_due = self
            .scheduled_frontier
            .first_key_value()
            .and_then(|(tick, wakes)| {
                wakes
                    .iter()
                    .next()
                    .map(|(ordinal, wake_id)| (*tick, *ordinal, *wake_id))
            });
        let next_ready = self
            .ready_frontier
            .first_key_value()
            .map(|(ordinal, wake_id)| (*ordinal, *wake_id));

        TemporalFrontierSnapshot::new(
            self.scheduled_frontier.len(),
            self.ready_frontier.len(),
            next_due.map(|(tick, _, _)| tick),
            next_due.map(|(_, _, wake_id)| wake_id),
            next_due.map(|(_, ordinal, _)| ordinal),
            next_ready.map(|(_, wake_id)| wake_id),
            next_ready.map(|(ordinal, _)| ordinal),
        )
    }
}

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn temporal_wake_summary(&self) -> TemporalWakeSummary {
        self.assert_construction_state_access();
        self.temporal.wake_summary()
    }

    pub fn temporal_frontier_snapshot(&self) -> TemporalFrontierSnapshot {
        self.assert_construction_state_access();
        self.temporal.frontier_snapshot()
    }
}
