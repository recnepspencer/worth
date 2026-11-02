//! Passive receiver work and elapsed telemetry for one named phase.
use std::time::{Duration, Instant};
use worth_query_host::facade::primary_graph::{
    advancement_observations_on_this_thread_for_test, WorthQueryExecutionObservation,
};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Journal {
    pub visits: [u64; 5],
    pub frontier: u64,
    pub retention_work: u64,
    pub requested_bytes: u64,
    pub backed_bytes: u64,
}

impl Journal {
    fn since(self, before: Self) -> Self {
        Self {
            visits: std::array::from_fn(|index| self.visits[index] - before.visits[index]),
            frontier: self.frontier - before.frontier,
            retention_work: self.retention_work - before.retention_work,
            requested_bytes: self.requested_bytes - before.requested_bytes,
            backed_bytes: self.backed_bytes - before.backed_bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Cost {
    pub journal: Journal,
    pub native_work: u64,
    pub peak_charged_bytes: u64,
    pub elapsed: Duration,
}

impl Cost {
    pub fn accumulate(&mut self, other: Self) {
        for index in 0..5 {
            self.journal.visits[index] += other.journal.visits[index];
        }
        self.journal.frontier += other.journal.frontier;
        self.journal.retention_work += other.journal.retention_work;
        self.journal.requested_bytes += other.journal.requested_bytes;
        self.journal.backed_bytes += other.journal.backed_bytes;
        self.native_work += other.native_work;
        self.peak_charged_bytes = self.peak_charged_bytes.max(other.peak_charged_bytes);
        self.elapsed += other.elapsed;
    }
}

pub(super) struct Meter {
    observation: WorthQueryExecutionObservation,
    before: Journal,
    started: Instant,
}

impl Meter {
    pub fn start(before: Journal) -> Self {
        let observation = WorthQueryExecutionObservation::install_on_this_thread();
        let _ = advancement_observations_on_this_thread_for_test();
        Self {
            observation,
            before,
            started: Instant::now(),
        }
    }

    pub fn finish(self, after: Journal) -> Cost {
        let elapsed = self.started.elapsed();
        let native = advancement_observations_on_this_thread_for_test();
        assert!(native.iter().all(|entry| entry.refusal.is_none()));
        let cost = Cost {
            journal: after.since(self.before),
            native_work: native.iter().map(|entry| entry.report.charged_work()).sum(),
            peak_charged_bytes: native
                .iter()
                .map(|entry| entry.report.physical().peak_charged_memory_bytes())
                .max()
                .unwrap_or(0),
            elapsed,
        };
        drop(self.observation);
        cost
    }
}
