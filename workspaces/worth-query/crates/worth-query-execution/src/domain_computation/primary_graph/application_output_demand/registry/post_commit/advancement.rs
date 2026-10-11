//! Release the row's resources after installing its advancement outcome.
use super::super::refreshed_rejoin::infallible;
use super::walk_charge::PostCommitWalk;
use super::*;
pub(in super::super) fn release_completed_prerequisites(
    state: &mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) -> Vec<Arc<WorthQueryOutputDemandKey>> {
    infallible(state.release_record_prerequisites(key, &mut PostCommitWalk::after_effect()))
}
pub(in super::super) fn release_completed_member(
    state: &mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) {
    infallible(state.remove_required_member_if_released(key, &mut PostCommitWalk::after_effect()));
}
