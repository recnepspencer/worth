//! A trusted wall clock the host holds at a chosen instant and moves only on
//! purpose, so admission time is deterministic in tests and simulations.
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, SystemTime};

use worth_foundational::facade::CanonicalTimestamp;
use worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock;

use super::time_basis::WorthQueryRuntimeTimeDenial;
use super::time_source::{WorthQueryRuntimeTimeSource, WorthQueryRuntimeTimeSourceDenial};

/// A runtime time source that reports one held instant until the host sets or
/// advances it. Clones share the instant: keep one clone to move time after
/// handing another to `WorthQueryHomeConfig::with_authorization_time_source`.
///
/// The monotonic admission clock (request deadlines, session lifetimes) is a
/// separate timeline; it is the system clock unless
/// [`Self::with_admission_clock`] supplies the Home's own observer.
#[derive(Clone, Debug)]
pub struct WorthQueryFixedRuntimeTimeSource {
    now: Arc<Mutex<CanonicalTimestamp>>,
    admission_clock: WorthQueryAdmissionClock,
}

impl WorthQueryFixedRuntimeTimeSource {
    /// Holds trusted time at `at`.
    pub fn new(at: CanonicalTimestamp) -> Self {
        Self {
            now: Arc::new(Mutex::new(at)),
            admission_clock: WorthQueryAdmissionClock::system(),
        }
    }

    /// Reports `clock` as the monotonic admission observer; pass the same
    /// clock given to `WorthQueryHomeConfig::with_clock`.
    pub fn with_admission_clock(mut self, clock: WorthQueryAdmissionClock) -> Self {
        self.admission_clock = clock;
        self
    }

    /// The instant currently held.
    pub fn now(&self) -> CanonicalTimestamp {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Holds trusted time at `at`, earlier or later than before.
    pub fn set(&self, at: CanonicalTimestamp) {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner) = at;
    }

    /// Moves the held instant forward by `elapsed` (truncated to whole
    /// microseconds) and returns it; an unrepresentable result is refused and
    /// leaves the held instant unchanged.
    pub fn advance(
        &self,
        elapsed: Duration,
    ) -> Result<CanonicalTimestamp, WorthQueryRuntimeTimeDenial> {
        let mut now = self.now.lock().unwrap_or_else(PoisonError::into_inner);
        let advanced = i64::try_from(elapsed.as_micros())
            .ok()
            .and_then(|micros| now.micros_since_unix_epoch.checked_add(micros))
            .ok_or(WorthQueryRuntimeTimeDenial::TimelineValueOverflow)?;
        *now = CanonicalTimestamp {
            micros_since_unix_epoch: advanced,
        };
        Ok(*now)
    }
}

impl WorthQueryRuntimeTimeSource for WorthQueryFixedRuntimeTimeSource {
    fn admission_clock(&self) -> WorthQueryAdmissionClock {
        self.admission_clock.clone()
    }

    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        super::canonical_instant::system_time_of(self.now())
            .ok_or(WorthQueryRuntimeTimeSourceDenial::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::super::time_basis::WorthQueryRuntimeClock;
    use super::*;

    fn at(micros: i64) -> CanonicalTimestamp {
        CanonicalTimestamp {
            micros_since_unix_epoch: micros,
        }
    }

    #[test]
    fn clones_share_the_held_instant_through_the_runtime_clock() {
        let held = WorthQueryFixedRuntimeTimeSource::new(at(1_795_829_400_000_000));
        let clock = WorthQueryRuntimeClock::from_source(held.clone());
        assert_eq!(clock.admission_timestamp(), Ok(at(1_795_829_400_000_000)));
        assert_eq!(clock.admission_timestamp(), Ok(at(1_795_829_400_000_000)));
        assert_eq!(
            held.advance(Duration::from_secs(19_800)),
            Ok(at(1_795_849_200_000_000))
        );
        assert_eq!(clock.admission_timestamp(), Ok(at(1_795_849_200_000_000)));
        held.set(at(-1));
        assert_eq!(clock.admission_timestamp(), Ok(at(-1)));
    }

    #[test]
    fn an_unrepresentable_advance_is_refused_and_keeps_the_instant() {
        let held = WorthQueryFixedRuntimeTimeSource::new(at(i64::MAX - 1));
        assert_eq!(
            held.advance(Duration::from_micros(2)),
            Err(WorthQueryRuntimeTimeDenial::TimelineValueOverflow)
        );
        assert_eq!(held.now(), at(i64::MAX - 1));
    }

    #[test]
    fn the_admission_observer_is_the_one_supplied() {
        let controller = worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionTimeController::new()
            .expect("a controlled admission clock is available");
        let held =
            WorthQueryFixedRuntimeTimeSource::new(at(0)).with_admission_clock(controller.clock());
        assert!(held.admission_clock().same_clock(&controller.clock()));
        assert!(WorthQueryFixedRuntimeTimeSource::new(at(0))
            .admission_clock()
            .same_clock(&WorthQueryAdmissionClock::system()));
    }
}
