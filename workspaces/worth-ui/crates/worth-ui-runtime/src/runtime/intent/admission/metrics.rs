#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiIntentAdmissionMetrics {
    active_attempts: usize,
    active_occupancy: usize,
    retained_candidates: usize,
    retained_payloads: usize,
    retained_owner_references: usize,
    admitted: u64,
    released: u64,
    lifecycle_cancelled: u64,
    stopped: u64,
    operability_reobservations: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiIntentAdmissionShutdownReport {
    settled_attempts: usize,
    active_after: usize,
    retained_candidates_after: usize,
    retained_payloads_after: usize,
}

pub(super) struct UiIntentAdmissionMetricInput {
    pub(super) active_attempts: usize,
    pub(super) active_occupancy: usize,
    pub(super) retained_candidates: usize,
    pub(super) retained_payloads: usize,
    pub(super) retained_owner_references: usize,
    pub(super) admitted: u64,
    pub(super) released: u64,
    pub(super) lifecycle_cancelled: u64,
    pub(super) stopped: u64,
    pub(super) operability_reobservations: u64,
}

impl UiIntentAdmissionMetrics {
    pub(super) const fn new(input: UiIntentAdmissionMetricInput) -> Self {
        Self {
            active_attempts: input.active_attempts,
            active_occupancy: input.active_occupancy,
            retained_candidates: input.retained_candidates,
            retained_payloads: input.retained_payloads,
            retained_owner_references: input.retained_owner_references,
            admitted: input.admitted,
            released: input.released,
            lifecycle_cancelled: input.lifecycle_cancelled,
            stopped: input.stopped,
            operability_reobservations: input.operability_reobservations,
        }
    }

    pub const fn active_attempts(self) -> usize {
        self.active_attempts
    }

    pub const fn active_occupancy(self) -> usize {
        self.active_occupancy
    }

    pub const fn retained_candidates(self) -> usize {
        self.retained_candidates
    }

    pub const fn retained_payloads(self) -> usize {
        self.retained_payloads
    }

    pub const fn retained_owner_references(self) -> usize {
        self.retained_owner_references
    }

    pub const fn admitted(self) -> u64 {
        self.admitted
    }

    pub const fn released(self) -> u64 {
        self.released
    }

    pub const fn lifecycle_cancelled(self) -> u64 {
        self.lifecycle_cancelled
    }

    pub const fn stopped(self) -> u64 {
        self.stopped
    }

    /// Standing operability facts a condition change re-observed, one per
    /// attempted refresh of an existing fact.
    pub const fn operability_reobservations(self) -> u64 {
        self.operability_reobservations
    }
}

impl UiIntentAdmissionShutdownReport {
    pub(super) const fn new(settled_attempts: usize, metrics: UiIntentAdmissionMetrics) -> Self {
        Self {
            settled_attempts,
            active_after: metrics.active_attempts,
            retained_candidates_after: metrics.retained_candidates,
            retained_payloads_after: metrics.retained_payloads,
        }
    }

    pub const fn settled_attempts(self) -> usize {
        self.settled_attempts
    }

    pub const fn active_after(self) -> usize {
        self.active_after
    }

    pub const fn retained_candidates_after(self) -> usize {
        self.retained_candidates_after
    }

    pub const fn retained_payloads_after(self) -> usize {
        self.retained_payloads_after
    }
}
