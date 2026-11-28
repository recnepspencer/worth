use std::cmp::Ordering;
use std::sync::Arc;
use std::time::Duration;

use worth_execution::time::TimeSampler;
use worth_foundational::time::{BootEpoch, MonotonicInstant};

/// Monotonic value supplied by the explicit Runtime World clock port.
///
/// The value is meaningful only for deadline and cleanup eligibility. It is
/// deliberately absent from identities, bases, parentage, and outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeWorldInstant(MonotonicInstant);

impl RuntimeWorldInstant {
    /// Construct a coordinate for a caller-supplied tick clock whose source
    /// and consumers agree on one non-rebooting timeline.
    pub const fn from_ticks(ticks: u64) -> Self {
        Self(MonotonicInstant::new(ticks, BootEpoch::new(0)))
    }

    pub const fn from_monotonic(instant: MonotonicInstant) -> Self {
        Self(instant)
    }

    pub fn checked_add(self, duration: Duration) -> Option<Self> {
        self.0.checked_add(duration).map(Self)
    }

    pub(crate) fn elapsed_ticks_since(self, earlier: Self) -> Option<u64> {
        self.0
            .checked_duration_since(earlier.0)
            .ok()?
            .as_nanos()
            .try_into()
            .ok()
    }

    /// A deadline on an incomparable timeline cannot authorize more work.
    pub(crate) fn deadline_elapsed(self, deadline: Self) -> bool {
        self.0.cmp_same_epoch(deadline.0) != Ok(Ordering::Less)
    }
}

/// Source for the purpose-specific monotonic clock. A clock is an allowed
/// runtime dependency, not a component-owner adapter. Implementations must be
/// monotonic, bounded, nonblocking, and must not re-enter Runtime World. The
/// final deadline check runs under the exact product-cell guard before movement;
/// no clock callback runs after that irreversible cutoff.
pub trait RuntimeWorldClockSource: Send + Sync {
    fn now(&self) -> RuntimeWorldInstant;
}

impl RuntimeWorldClockSource for TimeSampler {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_monotonic(self.monotonic())
    }
}

/// Cloneable clock port held by the future managed Runtime World owner.
#[derive(Clone)]
pub struct RuntimeWorldClock {
    source: Arc<dyn RuntimeWorldClockSource>,
}

impl std::fmt::Debug for RuntimeWorldClock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeWorldClock")
            .finish_non_exhaustive()
    }
}

impl RuntimeWorldClock {
    pub fn from_source<S>(source: S) -> Self
    where
        S: RuntimeWorldClockSource + 'static,
    {
        Self {
            source: Arc::new(source),
        }
    }

    pub fn now(&self) -> RuntimeWorldInstant {
        self.source.now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_execution::time::ControlledTime;

    #[test]
    fn sampled_deadlines_expire_at_the_cutoff_and_after_reboot() {
        let time = Arc::new(ControlledTime::at_unix_secs(0));
        let clock = RuntimeWorldClock::from_source(TimeSampler::new(time.clone()));
        let start = clock.now();
        let duration = Duration::from_nanos(5);
        let deadline = start.checked_add(duration).unwrap();
        assert!(!start.deadline_elapsed(deadline));
        time.advance(duration);
        assert!(clock.now().deadline_elapsed(deadline));
        assert_eq!(clock.now().elapsed_ticks_since(start), Some(5));

        let not_yet_elapsed = clock.now().checked_add(duration).unwrap();
        time.reboot_epoch();
        assert!(clock.now().deadline_elapsed(not_yet_elapsed));
        assert_eq!(clock.now().elapsed_ticks_since(start), None);
    }
}
