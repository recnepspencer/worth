use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::domain_computation::runtime_time::{
    WorthQueryRuntimeTimeSource, WorthQueryRuntimeTimeSourceDenial,
};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct AuthorizationTimeController {
    state: Arc<Mutex<AuthorizationTimeState>>,
    pub(super) admission_clock:
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
}

#[derive(Default)]
enum AuthorizationTimeState {
    #[default]
    System,
    Held(SystemTime),
    Scripted(VecDeque<SystemTime>),
}

impl AuthorizationTimeController {
    /// Return one explicit deterministic owner sample for every observation.
    /// Use `script` when sample order or exhaustion is part of the test.
    pub(in crate::domain_computation::primary_graph) fn hold(&self, sample: SystemTime) {
        *self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            AuthorizationTimeState::Held(sample);
    }

    pub(in crate::domain_computation::primary_graph) fn script(
        &self,
        samples: impl IntoIterator<Item = SystemTime>,
    ) {
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
