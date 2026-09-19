use std::time::Instant;

use worth_runtime_world::facade::{RuntimeWorldClockSource, RuntimeWorldInstant};

/// Query's monotonic clock for one installed Product World.
///
/// Construction is explicit so every public runtime root states which clock
/// defines its deadlines and recovery ages.
#[derive(Clone)]
pub struct WorthQueryProductWorldClock {
    origin: Instant,
}

impl WorthQueryProductWorldClock {
    pub fn start() -> Self {
        Self {
            origin: Instant::now(),
        }
    }

    pub(crate) fn deadline(&self, deadline: Instant) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(
            deadline
                .saturating_duration_since(self.origin)
                .as_nanos()
                .min(u128::from(u64::MAX)) as u64,
        )
    }
}

impl RuntimeWorldClockSource for WorthQueryProductWorldClock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(
            self.origin.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
        )
    }
}
