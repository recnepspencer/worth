//! Admission observer installed beside the checkpoint fixture's wall-time source.
use super::*;

pub(super) struct FixtureAdmissionTime(pub(super) authentication::WorthQueryAdmissionClock);

impl primary_graph::WorthQueryRuntimeTimeSource for FixtureAdmissionTime {
    fn admission_clock(&self) -> authentication::WorthQueryAdmissionClock {
        self.0.clone()
    }
    fn current_time(
        &self,
    ) -> Result<std::time::SystemTime, primary_graph::WorthQueryRuntimeTimeSourceDenial> {
        Ok(std::time::SystemTime::now())
    }
}
