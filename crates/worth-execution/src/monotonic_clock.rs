use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
pub enum MonotonicTimeConstructionDenial { IdentityExhausted }

#[derive(Debug)]
enum ClockState {
    Running,
    Held(Instant),
}

impl MonotonicClock {
    pub const fn system() -> Self {
        Self { controlled: None }
    }

    pub fn now(&self) -> Instant {
        match &self.controlled {
            None => Instant::now(),
            Some(state) => match *state.state.lock().expect("admission clock poisoned") {
                ClockState::Running => Instant::now(),
                ClockState::Held(now) => now,
            },
        }
    }

    pub(crate) fn identity(&self) -> u64 {
        match &self.controlled { None => 0, Some(clock) => clock.identity }
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
        let mut next = NEXT_CLOCK_IDENTITY.lock().expect("clock identity owner poisoned");
        let following = next.checked_add(1).ok_or(MonotonicTimeConstructionDenial::IdentityExhausted)?;
        let identity = *next;
        *next = following;
        Ok(Self { state: Arc::new(ControlledClock { identity, state: Mutex::new(ClockState::Running) }) })
    }

    pub fn clock(&self) -> MonotonicClock {
        MonotonicClock {
            controlled: Some(Arc::clone(&self.state)),
        }
    }

    pub fn hold(&self) -> Instant {
        let mut state = self.state.state.lock().expect("admission clock poisoned");
        match *state {
            ClockState::Running => {
                let now = Instant::now();
                *state = ClockState::Held(now);
                now
            }
            ClockState::Held(now) => now,
        }
    }

    pub fn advance(&self, elapsed: Duration) -> Result<Instant, MonotonicTimeAdvanceDenial> {
        let mut state = self.state.state.lock().expect("admission clock poisoned");
        let ClockState::Held(now) = *state else {
            return Err(MonotonicTimeAdvanceDenial::NotHeld);
        };
        let advanced = now
            .checked_add(elapsed)
            .ok_or(MonotonicTimeAdvanceDenial::InstantOverflow)?;
        *state = ClockState::Held(advanced);
        Ok(advanced)
    }
}
