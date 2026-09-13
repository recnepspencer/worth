use std::path::Path;

use super::counters::OwnerSemanticVerificationCounters;
use super::outcome::{OwnerSemanticVerificationDenial, OwnerSemanticVerificationResourceDenial};
use crate::backup_verification::owner_resource_budget::{
    maximum_requested_owned_allocation_bytes, maximum_reserved_owned_allocation_bytes,
};
use crate::inspection::OwnerDecodedArtifactBinding;
use crate::truth_composition::RecoveryCandidateObservation;
use worth_store_physical_format::BackupBundleManifest;

pub(super) struct OwnerAdmission {
    pub(super) counters: OwnerSemanticVerificationCounters,
    pub(super) recovery_candidates: Vec<RecoveryCandidateObservation>,
    pub(super) owner_bindings: Vec<OwnerDecodedArtifactBinding>,
    pub(super) expected_root: Option<worth_store_physical_format::RootPublicationCell>,
}

pub(super) fn admit(
    root: &Path,
    manifest: &BackupBundleManifest,
    max_buffer_bytes: usize,
    maximum_owned_allocation_bytes: u64,
) -> Result<OwnerAdmission, OwnerSemanticVerificationDenial> {
    let requested = maximum_requested_owned_allocation_bytes(root, manifest)
        .and_then(|bytes| bytes.checked_add(max_buffer_bytes as u64))
        .unwrap_or(u64::MAX);
    if requested > maximum_owned_allocation_bytes {
        return Err(OwnerSemanticVerificationDenial::Resource(
            OwnerSemanticVerificationResourceDenial {
                required_bytes: requested,
                limit_bytes: maximum_owned_allocation_bytes,
            },
        ));
    }
    let mut recovery_candidates = Vec::new();
    let mut owner_bindings = Vec::new();
    if recovery_candidates
        .try_reserve_exact(manifest.artifacts().len())
        .is_err()
        || owner_bindings
            .try_reserve_exact(manifest.artifacts().len().saturating_add(1))
            .is_err()
    {
        return Err(OwnerSemanticVerificationDenial::AllocationFailed);
    }
    let reserved_peak = maximum_reserved_owned_allocation_bytes(
        root,
        manifest,
        recovery_candidates.capacity(),
        owner_bindings.capacity(),
        max_buffer_bytes,
    )
    .unwrap_or(u64::MAX);
    if reserved_peak > maximum_owned_allocation_bytes {
        return Err(OwnerSemanticVerificationDenial::Resource(
            OwnerSemanticVerificationResourceDenial {
                required_bytes: reserved_peak,
                limit_bytes: maximum_owned_allocation_bytes,
            },
        ));
    }
    Ok(OwnerAdmission {
        counters: OwnerSemanticVerificationCounters::default(),
        recovery_candidates,
        owner_bindings,
        expected_root: None,
    })
}
