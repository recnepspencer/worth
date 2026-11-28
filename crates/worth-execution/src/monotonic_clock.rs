use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use worth_foundational::time::MonotonicInstant;

use crate::time::{system_deadline_now, ControlledTime, TimeSampler};

/// Observe-only monotonic time retained by one installed admission world.
/// System observers share the real monotonic timeline; controlled worlds have
/// distinct, memory-only identities that never enter evidence or content hashes.
#[derive(Clone, Debug)]
pub struct MonotonicClock {
    controlled: Option<Arc<ControlledClock>>,
}

#[derive(Debug)]
struct ControlledClock {
    // Memory-only ordering of clock owners; never persist, hash or record it as evidence.
    identity: u64,
    state: Mutex<ClockState>,
}

static NEXT_CLOCK_IDENTITY: Mutex<u64> = Mutex::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonotonicTimeConstructionDenial {
    IdentityExhausted,
}

enum ClockState {
    Running,
    Held(HeldClock),
}

struct HeldClock {
    origin: Instant,
    start: MonotonicInstant,
    control: Arc<ControlledTime>,
    sampler: TimeSampler,
}

impl HeldClock {
    fn new() -> Self {
        let origin = system_deadline_now();
        let control = Arc::new(ControlledTime::at_unix_secs(0));
        let sampler = TimeSampler::new(control.clone());
        let start = sampler.monotonic();
        Self {
            origin,
            start,
            control,
            sampler,
        }
    }

    fn elapsed(&self) -> Duration {
        self.sampler
            .monotonic()
            .checked_duration_since(self.start)
            .expect("the controller retains a single boot")
    }

    fn now(&self) -> Instant {
        self.origin + self.elapsed()
    }
}

impl std::fmt::Debug for ClockState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => f.write_str("Running"),
            Self::Held(held) => f.debug_tuple("Held").field(&held.now()).finish(),
        }
    }
}

impl MonotonicClock {
    pub const fn system() -> Self {
        Self { controlled: None }
    }

    pub fn now(&self) -> Instant {
        match &self.controlled {
            None => system_deadline_now(),
            Some(state) => match &*state.state.lock().expect("admission clock poisoned") {
                ClockState::Running => system_deadline_now(),
                ClockState::Held(held) => held.now(),
            },
        }
    }

    pub(crate) fn identity(&self) -> u64 {
        match &self.controlled {
            None => 0,
            Some(clock) => clock.identity,
        }
    }

    pub fn same_clock(&self, other: &Self) -> bool {
        match (&self.controlled, &other.controlled) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, Some(_)) | (Some(_), None) => false,
        }
    }
}

/// The world's time owner. Admission callers receive only its observer.
/// Holding is irreversible, so existing deadlines cannot be made fresh again.
#[derive(Clone, Debug)]
pub struct MonotonicTimeController {
    state: Arc<ControlledClock>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonotonicTimeAdvanceDenial {
    NotHeld,
    InstantOverflow,
}

impl MonotonicTimeController {
    pub fn new() -> Result<Self, MonotonicTimeConstructionDenial> {
        let mut next = NEXT_CLOCK_IDENTITY
            .lock()
            .expect("clock identity owner poisoned");
        let following = next
            .checked_add(1)
            .ok_or(MonotonicTimeConstructionDenial::IdentityExhausted)?;
        let identity = *next;
        *next = following;
        Ok(Self {
            state: Arc::new(ControlledClock {
                identity,
                state: Mutex::new(ClockState::Running),
            }),
        })
    }

    pub fn clock(&self) -> MonotonicClock {
        MonotonicClock {
            controlled: Some(Arc::clone(&self.state)),
        }
    }

    pub fn hold(&self) -> Instant {
        let mut state = self.state.state.lock().expect("admission clock poisoned");
        match &*state {
            ClockState::Running => {
                let held = HeldClock::new();
                let now = held.origin;
                *state = ClockState::Held(held);
                now
            }
            ClockState::Held(held) => held.now(),
        }
    }

    pub fn advance(&self, elapsed: Duration) -> Result<Instant, MonotonicTimeAdvanceDenial> {
        let state = self.state.state.lock().expect("admission clock poisoned");
        let ClockState::Held(held) = &*state else {
            return Err(MonotonicTimeAdvanceDenial::NotHeld);
        };
        let advanced = held
            .elapsed()
            .checked_add(elapsed)
            .filter(|total| u64::try_from(total.as_nanos()).is_ok())
            .and_then(|total| held.origin.checked_add(total))
            .ok_or(MonotonicTimeAdvanceDenial::InstantOverflow)?;
        held.control.advance(elapsed);
        Ok(advanced)
    }
}
