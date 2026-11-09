use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use worth_query_host::facade::primary_graph::{
    WorthQueryRuntimeTimeSource, WorthQueryRuntimeTimeSourceDenial,
};

#[derive(Clone)]
pub(crate) struct AuthorizationTimeController {
    current: Arc<Mutex<SystemTime>>,
    admission_clock:
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock,
}

impl AuthorizationTimeController {
    pub(crate) fn at_epoch_seconds(seconds: u64) -> Self {
        Self {
            current: Arc::new(Mutex::new(UNIX_EPOCH + Duration::from_secs(seconds))),
            admission_clock: worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        }
    }

    pub(crate) fn advance_to_epoch_seconds(&self, seconds: u64) {
        *self
            .current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            UNIX_EPOCH + Duration::from_secs(seconds);
    }
}

impl WorthQueryRuntimeTimeSource for AuthorizationTimeController {
    fn admission_clock(
        &self,
    ) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock
    {
        self.admission_clock.clone()
    }

    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        Ok(*self
            .current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()))
    }
}
