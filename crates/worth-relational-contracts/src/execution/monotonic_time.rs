//! Relational deadlines and measurement use Execution's monotonic clock.
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use worth_execution::time::{SystemClockAdapter, TimeSampler};
use worth_foundational::time::{ClockSourceIdentity, MonotonicInstant};

static SYSTEM_CLOCK: LazyLock<TimeSampler> = LazyLock::new(|| {
    TimeSampler::new(Arc::new(SystemClockAdapter::new(ClockSourceIdentity::new(
        0,
    ))))
});

pub fn monotonic_now() -> MonotonicInstant {
    SYSTEM_CLOCK.monotonic()
}

pub fn elapsed_since(started: MonotonicInstant) -> Duration {
    monotonic_now()
        .checked_duration_since(started)
        .expect("Relational system telemetry shares one monotonic clock")
}

/// An additional, boot-local deadline sampled by its originating clock.
/// A boot change expires the deadline because its basis can no longer be used.
#[derive(Clone, Copy)]
pub struct RelationalCutoverDeadline<'clock> {
    clock: &'clock TimeSampler,
    started: MonotonicInstant,
    duration: Duration,
}

impl<'clock> RelationalCutoverDeadline<'clock> {
    pub fn after(clock: &'clock TimeSampler, duration: Duration) -> Self {
        Self {
            clock,
            started: clock.monotonic(),
            duration,
        }
    }

    pub fn system_after(duration: Duration) -> RelationalCutoverDeadline<'static> {
        Self::with_system_start(monotonic_now(), duration)
    }

    /// Adapt a caller deadline on the original OS timeline at the API boundary.
    pub fn from_system_instant(instant: std::time::Instant) -> RelationalCutoverDeadline<'static> {
        let started = monotonic_now();
        let remaining =
            instant.saturating_duration_since(worth_execution::MonotonicClock::system().now());
        Self::with_system_start(started, remaining)
    }

    fn with_system_start(
        started: MonotonicInstant,
        duration: Duration,
    ) -> RelationalCutoverDeadline<'static> {
        RelationalCutoverDeadline {
            clock: &SYSTEM_CLOCK,
            started,
            duration,
        }
    }

    /// Strictly positive room on the originating boot timeline.
    pub fn remaining(self) -> Option<Duration> {
        let elapsed = self
            .clock
            .monotonic()
            .checked_duration_since(self.started)
            .ok()?;
        self.duration
            .checked_sub(elapsed)
            .filter(|remaining| !remaining.is_zero())
    }

    pub fn is_elapsed(self) -> bool {
        self.remaining().is_none()
    }
}
