mod counters;
mod event;

pub use counters::WorthQueryConvergenceEpochCounters;
pub use event::{WorthQueryConvergenceAdmissionStartEvent, WorthQueryConvergenceLifecycleEvent};

pub struct WorthQueryConvergenceEpochLifecycle {
    counters: WorthQueryConvergenceEpochCounters,
}

impl WorthQueryConvergenceEpochLifecycle {
    pub fn begin<E>(event: E) -> Self
    where
        E: WorthQueryConvergenceAdmissionStartEvent,
    {
        let mut lifecycle = Self {
            counters: WorthQueryConvergenceEpochCounters::empty(),
        };
        lifecycle.record(event);
        lifecycle
    }

    pub fn record<E>(&mut self, event: E)
    where
        E: WorthQueryConvergenceLifecycleEvent,
    {
        event.apply(&mut self.counters);
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        &self.counters
    }

    pub fn into_counters(self) -> WorthQueryConvergenceEpochCounters {
        self.counters
    }
}
