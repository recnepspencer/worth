use core::time::Duration;
use std::sync::Mutex;

use worth_foundational::time::{BootEpoch, ClockSourceIdentity, TimeUnavailable};

use super::{boot::next_boot, RawClockAdapter};

/// Deterministic clock owned by a test or simulation. Time moves only through
/// this receiver's controls. Each clock and reboot gets a distinct boot epoch.
pub struct ControlledTime {
    state: Mutex<ClockState>,
    source: ClockSourceIdentity,
}

struct ClockState {
    wall: i128,
    ticks: u64,
    boot: BootEpoch,
}

impl ControlledTime {
    /// Starts with a fresh process-local source identity and zero ticks.
    pub fn at_unix_secs(unix_secs: u64) -> Self {
        let (boot, id) = next_boot();
        Self::start(unix_secs, ClockSourceIdentity::new(id), boot)
    }

    /// Starts a fresh boot with an explicitly assigned logical wall source.
    /// A simulation can reuse the assignment when reproducing recorded evidence.
    pub fn with_source_identity(unix_secs: u64, source: ClockSourceIdentity) -> Self {
        Self::start(unix_secs, source, next_boot().0)
    }

    fn start(unix_secs: u64, source: ClockSourceIdentity, boot: BootEpoch) -> Self {
        Self {
            state: Mutex::new(ClockState {
                wall: i128::from(unix_secs) * 1_000_000_000,
                ticks: 0,
                boot,
            }),
            source,
        }
    }

    /// Advances both coordinates. Panics before changing either coordinate if
    /// the duration would exhaust the boot's nanosecond counter.
    pub fn advance(&self, delta: Duration) {
        let mut state = self.state.lock().expect("controlled clock poisoned");
        let ticks = u64::try_from(delta.as_nanos())
            .ok()
            .and_then(|nanos| state.ticks.checked_add(nanos));
        let Some(ticks) = ticks else {
            drop(state);
            panic!("controlled clock boot ticks exhausted");
        };
        state.wall += delta.as_nanos() as i128;
        state.ticks = ticks;
    }

    /// Steps wall time backward without changing the monotonic coordinate.
    /// Wall reads outside the Unix nanosecond range refuse until time recovers.
    pub fn step_wall_back(&self, delta: Duration) {
        let mut state = self.state.lock().expect("controlled clock poisoned");
        state.wall -= delta.as_nanos() as i128;
    }

    /// Starts a fresh boot with zero ticks and keeps the wall coordinate.
    pub fn reboot_epoch(&self) {
        let mut state = self.state.lock().expect("controlled clock poisoned");
        state.boot = next_boot().0;
        state.ticks = 0;
    }
}

impl RawClockAdapter for ControlledTime {
    fn read_wall_unix_nanos(&self) -> Result<u64, TimeUnavailable> {
        let state = self.state.lock().expect("controlled clock poisoned");
        u64::try_from(state.wall).map_err(|_| TimeUnavailable::WallOutOfRange)
    }

    fn read_monotonic_ticks(&self) -> (u64, BootEpoch) {
        let state = self.state.lock().expect("controlled clock poisoned");
        (state.ticks, state.boot)
    }

    fn source_identity(&self) -> ClockSourceIdentity {
        self.source
    }
}

#[cfg(test)]
mod tests;
