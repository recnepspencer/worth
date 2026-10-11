//! The completed backing transfer releases the prior row's membership.
use super::super::refreshed_rejoin::infallible;
use super::walk_charge::PostCommitWalk;
use super::*;
pub(in super::super) fn release_replaced_member(
    state: &mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) {
    infallible(state.remove_required_member_if_released(key, &mut PostCommitWalk::after_effect()));
}
