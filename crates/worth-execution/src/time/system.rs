use std::{
    sync::{Arc, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use worth_foundational::time::{BootEpoch, ClockSourceIdentity, TimeUnavailable};

use super::{boot::next_boot, RawClockAdapter, TimeSampler};

struct SystemTimeBasis {
    sampler: Arc<TimeSampler>,
    origin: Instant,
    boot: BootEpoch,
}

fn system_time_basis() -> &'static SystemTimeBasis {
    static SYSTEM: OnceLock<SystemTimeBasis> = OnceLock::new();
    SYSTEM.get_or_init(|| {
        let adapter = SystemClockAdapter::new(ClockSourceIdentity::new(0));
        SystemTimeBasis {
            origin: adapter.origin,
            boot: adapter.boot,
            sampler: Arc::new(TimeSampler::new(Arc::new(adapter))),
        }
    })
}

/// The process's shared default OS time sampler.
pub fn system_time_sampler() -> &'static Arc<TimeSampler> {
    &system_time_basis().sampler
}

/// Projects shared monotonic ticks onto the OS timeline for deadline APIs.
pub(crate) fn system_deadline_now() -> Instant {
    let basis = system_time_basis();
    let origin = worth_foundational::time::MonotonicInstant::new(0, basis.boot);
    basis.origin
        + basis
            .sampler
            .monotonic()
            .checked_duration_since(origin)
            .expect("the OS deadline timeline keeps its boot identity")
}

/// Execution's OS wall and monotonic clock boundary. Assign the wall source
/// identity at composition. Each adapter owns its own monotonic origin.
pub struct SystemClockAdapter {
    source: ClockSourceIdentity,
    origin: Instant,
    boot: BootEpoch,
}

impl SystemClockAdapter {
    pub fn new(source: ClockSourceIdentity) -> Self {
        Self {
            source,
            origin: Self::read_os_monotonic(),
            boot: next_boot().0,
        }
    }

    // Admission deadlines still consume an OS Instant through this boundary.
    #[expect(
        clippy::disallowed_methods,
        reason = "Execution owns OS monotonic reads"
    )]
    fn read_os_monotonic() -> Instant {
        Instant::now()
    }
}

impl RawClockAdapter for SystemClockAdapter {
    #[expect(
        clippy::disallowed_methods,
        reason = "Execution owns OS Unix wall reads"
    )]
    fn read_wall_unix_nanos(&self) -> Result<u64, TimeUnavailable> {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TimeUnavailable::WallOutOfRange)?;
        u64::try_from(elapsed.as_nanos()).map_err(|_| TimeUnavailable::WallOutOfRange)
    }

    fn read_monotonic_ticks(&self) -> (u64, BootEpoch) {
        let elapsed: Duration = Self::read_os_monotonic().duration_since(self.origin);
        let ticks = u64::try_from(elapsed.as_nanos()).expect("OS clock boot ticks exhausted");
        (ticks, self.boot)
    }

    fn source_identity(&self) -> ClockSourceIdentity {
        self.source
    }
}
