//! Existing lane and phase layouts; no witness is minted.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    let _ = census.owner::<WorthQuerySharedLiveProjectionLease<(), (), (), crate::basis_lifecycle::ObservationLaneWitness>>("workspaces/worth-query/crates/worth-query/src/domain_installation/operation_execution/projection_sharing/lease.rs", "WorthQuerySharedLiveProjectionLease", 128 * 1024);
}
