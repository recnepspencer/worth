//! Existing lane and phase layouts; no witness is minted.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    let _ = census.owner::<WorthQueryOperationalProjection<crate::domain_installation::WorthQuerySettledDomainProjection<(), (), (), crate::basis_lifecycle::ObservationLaneWitness>, crate::basis_lifecycle::ObservationLaneWitness, super::super::states::WorthQueryLiveProjectionPhase>>("workspaces/worth-query/crates/worth-query/src/domain_installation/operation_execution/projection_lifecycle/operational_owner.rs", "WorthQueryOperationalProjection", 128 * 1024);
}
