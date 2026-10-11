//! Exact published authorities identify the owning completed publication.
use super::super::refreshed_rejoin::infallible;
use super::walk_charge::PostCommitWalk;
use super::*;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn release_published_member(
    state: &mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
    published: &RecordedSettlementIdentity,
) {
    infallible(state.remove_required_member_if_released(
        key,
        &mut PostCommitWalk::after_publication(published),
    ));
}
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn release_settled_member(
    state: &mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
    accepted: &WorthQueryAcceptedOutputAuthority,
) {
    infallible(
        state.remove_required_member_if_released(
            key,
            &mut PostCommitWalk::after_settlement(accepted),
        ),
    );
}
