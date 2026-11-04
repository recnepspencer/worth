use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

use worth_query_execution::facade::domain_computation::WorthQueryRuntimeTimeSource;
use worth_query_execution::facade::domain_computation::WorthQueryRuntimeTimeSourceDenial;

#[derive(Clone)]
pub struct AuthorizationTimeController {
    pub state: Arc<Mutex<AuthorizationTimeState>>,
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

impl WorthQueryRuntimeTimeSource for AuthorizationTimeController {
    fn admission_clock(
        &self,
    ) -> worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock {
        self.admission_clock.clone()
    }
    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &mut *state {
            AuthorizationTimeState::System => Ok(SystemTime::now()),
            AuthorizationTimeState::Held(sample) => Ok(*sample),
            AuthorizationTimeState::Scripted(samples) => samples
                .pop_front()
                .ok_or(WorthQueryRuntimeTimeSourceDenial::Unavailable),
        }
    }
}

impl Default for AuthorizationTimeController {
    fn default() -> Self {
        Self { state: Default::default(), admission_clock: worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system() }
    }
}
