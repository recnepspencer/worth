use std::sync::{Arc, Mutex};
use std::time::Duration;

use worth_query_host::facade::declaration::application_schema::CanonicalTimestamp;
use worth_query_host::facade::runtime::time::{ControlledTime, RawClockAdapter};
use worth_query_host::facade::runtime::{BootEpoch, ClockSourceIdentity, TimeUnavailable};

pub fn unix_nanos(at: CanonicalTimestamp) -> u64 {
    u64::try_from(at.micros_since_unix_epoch).unwrap() * 1_000
}

pub fn controlled_at(at: CanonicalTimestamp) -> Arc<ControlledTime> {
    Arc::new(ControlledTime::at_unix_nanos(unix_nanos(at)))
}

/// Advances one millisecond per wall read so a second read is detectable.
pub struct TickingClock {
    time: Mutex<ControlledTime>,
    source: ClockSourceIdentity,
}

impl TickingClock {
    pub fn new(at: CanonicalTimestamp) -> Arc<Self> {
        let time = ControlledTime::at_unix_nanos(unix_nanos(at));
        Arc::new(Self {
            source: time.source_identity(),
            time: Mutex::new(time),
        })
    }
}

impl RawClockAdapter for TickingClock {
    fn read_monotonic_ticks(&self) -> (u64, BootEpoch) {
        self.time.lock().unwrap().read_monotonic_ticks()
    }

    fn source_identity(&self) -> ClockSourceIdentity {
        self.source
    }

    fn read_wall_unix_nanos(&self) -> Result<u64, TimeUnavailable> {
        let time = self.time.lock().unwrap();
        let wall = time.read_wall_unix_nanos()?;
        time.advance(Duration::from_millis(1));
        Ok(wall)
    }
}
