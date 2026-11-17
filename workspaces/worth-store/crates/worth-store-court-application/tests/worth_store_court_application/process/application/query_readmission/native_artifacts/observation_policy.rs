//! Observational Source attachment has no mutation or maintenance admission.
use super::*;
#[derive(Debug)]
pub(super) struct ObservationMaintenanceMustRemainUnentered;
impl WorthQueryHomeMaintenance for ObservationMaintenanceMustRemainUnentered {
    fn after_advancement(&self, _: &ExecutionRequest<'_, '_>) {
        panic!("the observational attachment must never advance or schedule maintenance");
    }
}
