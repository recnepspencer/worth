//! Exact published authorities identify the owning completed publication.

use super::*;
pub(in super::super) fn release_published_member(
    state: &mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) {
    state.remove_required_member_if_released(key);
}
pub(in super::super) fn release_settled_member(
    state: &mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) {
    state.remove_required_member_if_released(key);
}
