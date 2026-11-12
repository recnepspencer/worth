//! Observational Source attachment has no mutation or maintenance admission.
use super::*;
#[derive(Debug)]
pub(super) struct ObservationMaintenanceMustRemainUnentered;
impl WorthQueryHomeMaintenance for ObservationMaintenanceMustRemainUnentered {
    fn after_advancement(&self, _: &ExecutionRequest<'_, '_>) {
        panic!("the observational attachment must never advance or schedule maintenance");
    }
}

/// This observational attachment has no permission to admit an operation.
pub(super) fn no_operation_admission(
    _: worth_foundational::facade::DurableStateFamily,
    _: u8,
) -> Option<u8> {
    panic!("the Bridge/source observation proof must never invoke Directory operation admission")
}
