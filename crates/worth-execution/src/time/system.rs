use std::time::{Instant, SystemTime, UNIX_EPOCH};

use worth_foundational::time::{BootEpoch, ClockSourceIdentity, TimeUnavailable};

use super::{boot::next_boot, RawClockAdapter};

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
    #[allow(
        clippy::disallowed_methods,
        reason = "Execution owns OS monotonic reads"
    )]
    pub(crate) fn read_os_monotonic() -> Instant {
        Instant::now()
    }
}

impl RawClockAdapter for SystemClockAdapter {
    #[allow(
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
        let elapsed = Self::read_os_monotonic().duration_since(self.origin);
        let ticks = u64::try_from(elapsed.as_nanos()).expect("OS clock boot ticks exhausted");
        (ticks, self.boot)
    }

    fn source_identity(&self) -> ClockSourceIdentity {
        self.source
    }
}
