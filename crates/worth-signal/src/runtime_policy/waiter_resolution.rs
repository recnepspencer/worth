//! Ordinary waiter-resolution work admission, independent of reconstruction.
use super::{InstalledSignalRuntimePolicy, ResolvedSignalRuntimePolicy, SignalRuntimePolicy};

pub(super) const DEFAULT_MAXIMUM_WAITER_RESOLUTION_VISITS: usize = 1_000_000;

impl SignalRuntimePolicy {
    /// Bound candidate visits, pending-membership scans, and recursive waiter
    /// resolution during one output preparation. Compilation rejects zero.
    /// This limit does not admit transient bytes or other output-preparation work.
    pub fn with_maximum_waiter_resolution_visits(mut self, maximum: usize) -> Self {
        self.maximum_waiter_resolution_visits = maximum;
        self
    }
}

impl ResolvedSignalRuntimePolicy {
    pub const fn maximum_waiter_resolution_visits(&self) -> usize {
        self.maximum_waiter_resolution_visits
    }
}

impl InstalledSignalRuntimePolicy {
    pub const fn maximum_waiter_resolution_visits(&self) -> usize {
        self.resolved().maximum_waiter_resolution_visits()
    }
}
