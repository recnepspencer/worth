use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use worth_execution::time::{system_time_sampler, ControlledTime, RawClockAdapter};
use worth_foundational::time::{BootEpoch, ClockSourceIdentity, TimeUnavailable};
use worth_query_execution::facade::primary_graph::WorthQueryRuntimeClock;

#[derive(Clone)]
pub struct AuthorizationTimeController {
    pub state: Arc<Mutex<AuthorizationTimeState>>,
    timeline: Arc<ControlledTime>,
    pub admission_clock:
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
}

#[derive(Default)]
pub enum AuthorizationTimeState {
    #[default]
    System,
    Held(SystemTime),
    Scripted(VecDeque<SystemTime>),
}

impl AuthorizationTimeController {
    pub fn on_clock(admission_clock: worth_execution::MonotonicClock) -> Self {
        Self {
            admission_clock,
            ..Default::default()
        }
    }

    /// Return one explicit deterministic owner sample for every observation.
    /// Use `script` when sample order or exhaustion is part of the test.
    pub fn hold(&self, sample: SystemTime) {
        *self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            AuthorizationTimeState::Held(sample);
    }

    pub fn script(&self, samples: impl IntoIterator<Item = SystemTime>) {
        *self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            AuthorizationTimeState::Scripted(samples.into_iter().collect());
    }
}

impl RawClockAdapter for AuthorizationTimeController {
    fn read_monotonic_ticks(&self) -> (u64, BootEpoch) {
        self.timeline.read_monotonic_ticks()
    }

    fn source_identity(&self) -> ClockSourceIdentity {
        self.timeline.source_identity()
    }

    fn read_wall_unix_nanos(&self) -> Result<u64, TimeUnavailable> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let wall = match &mut *state {
            AuthorizationTimeState::System => {
                return system_time_sampler().wall().map(|at| at.unix_nanos())
            }
            AuthorizationTimeState::Held(sample) => Ok(*sample),
            AuthorizationTimeState::Scripted(samples) => {
                samples.pop_front().ok_or(TimeUnavailable::WallUnavailable)
            }
        }?;
        wall.duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|delta| u64::try_from(delta.as_nanos()).ok())
            .ok_or(TimeUnavailable::WallOutOfRange)
    }
}

impl Default for AuthorizationTimeController {
    fn default() -> Self {
        Self {
            state: Default::default(),
            timeline: Arc::new(ControlledTime::at_unix_secs(0)),
            admission_clock: worth_execution::MonotonicClock::system(),
        }
    }
}

impl From<AuthorizationTimeController> for WorthQueryRuntimeClock {
    fn from(source: AuthorizationTimeController) -> Self {
        let clock = source.admission_clock.clone();
        Self::from(Arc::new(source)).with_admission_clock(clock)
    }
}
