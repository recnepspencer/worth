//! Trusted wall-time ingress paired with the already-owned admission observer.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    primary_graph::{WorthQueryRuntimeTimeSource, WorthQueryRuntimeTimeSourceDenial},
};

pub(crate) struct RuntimeTime(pub WorthQueryAdmissionClock);

impl WorthQueryRuntimeTimeSource for RuntimeTime {
    fn admission_clock(&self) -> WorthQueryAdmissionClock {
        self.0.clone()
    }

    fn current_time(&self) -> Result<std::time::SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        Ok(std::time::SystemTime::now())
    }
}
