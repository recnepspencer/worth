//! Release the row's resources after installing its advancement outcome.

use super::*;
pub(in super::super) fn release_completed_prerequisites(
    state: &mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) -> Vec<Arc<WorthQueryOutputDemandKey>> {
    // The scheduling/checkpoint outcome was committed to this row before this call.

    state.release_record_prerequisites(key)
}
pub(in super::super) fn release_completed_member(
    state: &mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) {
    // The scheduling/checkpoint outcome was committed to this row before this call.

    state.remove_required_member_if_released(key);
}
