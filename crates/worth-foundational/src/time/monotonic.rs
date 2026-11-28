use core::{cmp::Ordering, time::Duration};

use super::BootEpoch;

/// A nanosecond coordinate on a single boot-local monotonic timeline.
///
/// Compare through [Self::cmp_same_epoch] and use [Self::checked_duration_since]
/// for elapsed time. Keep these coordinates in memory.
///
/// ```compile_fail,E0369
/// use worth_foundational::time::{BootEpoch, MonotonicInstant};
///
/// let boot = BootEpoch::new(1);
/// assert!(MonotonicInstant::new(1, boot) < MonotonicInstant::new(2, boot));
/// ```
///
/// ```
/// use core::cmp::Ordering;
/// use worth_foundational::time::{BootEpoch, MonotonicInstant};
///
/// let boot = BootEpoch::new(1);
/// let first = MonotonicInstant::new(1, boot);
/// assert_eq!(first.cmp_same_epoch(MonotonicInstant::new(2, boot)), Ok(Ordering::Less));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MonotonicInstant {
    ticks: u64,
    boot: BootEpoch,
}

/// The two coordinates belong to distinct monotonic timelines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootEpochMismatch;

/// An elapsed duration cannot be formed from the supplied coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonotonicElapsedError {
    BootEpochMismatch(BootEpochMismatch),
    /// The earlier argument is later on the same timeline.
    EarlierIsLater,
}

impl MonotonicInstant {
    pub const fn new(ticks: u64, boot: BootEpoch) -> Self {
        Self { ticks, boot }
    }

    pub fn checked_add(self, duration: Duration) -> Option<Self> {
        let nanos = u64::try_from(duration.as_nanos()).ok()?;
        Some(Self::new(self.ticks.checked_add(nanos)?, self.boot))
    }

    pub fn cmp_same_epoch(self, other: Self) -> Result<Ordering, BootEpochMismatch> {
        self.check_epoch(other)?;
        Ok(self.ticks.cmp(&other.ticks))
    }

    pub fn checked_duration_since(self, earlier: Self) -> Result<Duration, MonotonicElapsedError> {
        self.check_epoch(earlier)
            .map_err(MonotonicElapsedError::BootEpochMismatch)?;
        self.ticks
            .checked_sub(earlier.ticks)
            .map(Duration::from_nanos)
            .ok_or(MonotonicElapsedError::EarlierIsLater)
    }

    fn check_epoch(self, other: Self) -> Result<(), BootEpochMismatch> {
        if self.boot == other.boot {
            Ok(())
        } else {
            Err(BootEpochMismatch)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elapsed_uses_checked_nanoseconds_only_within_one_boot() {
        for begin in [0, 1, u64::MAX - 1, u64::MAX] {
            for end in [0, 1, u64::MAX - 1, u64::MAX] {
                let first = MonotonicInstant::new(begin, BootEpoch::new(1));
                let second = MonotonicInstant::new(end, BootEpoch::new(1));
                assert_eq!(second.cmp_same_epoch(first), Ok(end.cmp(&begin)));
                let distance = (i128::from(end) - i128::from(begin)).try_into();
                assert_eq!(
                    second.checked_duration_since(first),
                    distance
                        .map(Duration::from_nanos)
                        .map_err(|_| MonotonicElapsedError::EarlierIsLater),
                );

                let foreign = MonotonicInstant::new(begin, BootEpoch::new(2));
                assert_eq!(second.cmp_same_epoch(foreign), Err(BootEpochMismatch));
                assert_eq!(
                    second.checked_duration_since(foreign),
                    Err(MonotonicElapsedError::BootEpochMismatch(BootEpochMismatch)),
                );
            }
        }
    }
}
