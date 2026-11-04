//! The body fixture's runtime owns the same monotonic observer as its requests.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    primary_graph::{WorthQueryRuntimeTimeSource, WorthQueryRuntimeTimeSourceDenial},
};

pub(super) struct FixtureAdmissionTime(pub(super) WorthQueryAdmissionClock);

impl WorthQueryRuntimeTimeSource for FixtureAdmissionTime {
    fn admission_clock(&self) -> WorthQueryAdmissionClock {
        self.0.clone()
    }

    fn current_time(&self) -> Result<std::time::SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        Ok(std::time::SystemTime::now())
    }
}
