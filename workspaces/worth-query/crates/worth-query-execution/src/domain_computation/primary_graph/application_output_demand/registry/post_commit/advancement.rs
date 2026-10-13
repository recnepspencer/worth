//! Release the row's resources after installing its advancement outcome.
use super::super::refreshed_rejoin::infallible;
use super::walk_charge::PostCommitWalk;
use super::*;
pub(in super::super) fn release_completed_prerequisites(
    state: &mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) -> Vec<Arc<WorthQueryOutputDemandKey>> {
    // The scheduling/checkpoint outcome was committed to this row before this call.
    let mut payer = PostCommitWalk::after_effect();
    infallible(state.release_record_prerequisites(key, &mut payer))
}
pub(in super::super) fn release_completed_member(
    state: &mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) {
    // The scheduling/checkpoint outcome was committed to this row before this call.
    let mut payer = PostCommitWalk::after_effect();
    infallible(state.remove_required_member_if_released(key, &mut payer));
}
