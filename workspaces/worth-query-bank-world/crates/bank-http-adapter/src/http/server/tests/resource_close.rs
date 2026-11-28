use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use worth_query_host::facade::runtime::time::{RawClockAdapter, SystemClockAdapter};
use worth_query_host::facade::runtime::{BootEpoch, ClockSourceIdentity, TimeUnavailable};

pub(super) struct TrackedAuthorizationTime {
    releases: Arc<AtomicUsize>,
    clock: SystemClockAdapter,
}

impl TrackedAuthorizationTime {
    pub(super) fn new(releases: Arc<AtomicUsize>) -> Self {
        Self {
            releases,
            clock: SystemClockAdapter::new(ClockSourceIdentity::new(0)),
        }
    }
}

impl RawClockAdapter for TrackedAuthorizationTime {
    fn read_monotonic_ticks(&self) -> (u64, BootEpoch) {
        self.clock.read_monotonic_ticks()
    }

    fn source_identity(&self) -> ClockSourceIdentity {
        self.clock.source_identity()
    }

    fn read_wall_unix_nanos(&self) -> Result<u64, TimeUnavailable> {
        self.clock.read_wall_unix_nanos()
    }
}

impl Drop for TrackedAuthorizationTime {
    fn drop(&mut self) {
        self.releases.fetch_add(1, Ordering::SeqCst);
    }
}
