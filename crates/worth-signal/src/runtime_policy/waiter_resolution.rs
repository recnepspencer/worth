//! Ordinary waiter-resolution work admission, independent of reconstruction.
use super::{InstalledSignalRuntimePolicy, ResolvedSignalRuntimePolicy, SignalRuntimePolicy};

impl SignalRuntimePolicy {
    /// Bound candidate visits, pending-membership scans, and recursive waiter
    /// resolution during one output preparation. None imposes no visit ceiling;
    /// compilation rejects an explicit zero.
    /// This limit does not admit transient bytes or other output-preparation work.
    pub fn with_maximum_waiter_resolution_visits(mut self, maximum: Option<usize>) -> Self {
        self.maximum_waiter_resolution_visits = maximum;
        self
    }
}

impl ResolvedSignalRuntimePolicy {
    pub const fn maximum_waiter_resolution_visits(&self) -> Option<usize> {
        self.maximum_waiter_resolution_visits
    }
}

impl InstalledSignalRuntimePolicy {
    pub const fn maximum_waiter_resolution_visits(&self) -> Option<usize> {
        self.resolved().maximum_waiter_resolution_visits()
    }
}
