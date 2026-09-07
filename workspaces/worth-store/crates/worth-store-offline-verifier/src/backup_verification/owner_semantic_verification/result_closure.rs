use std::path::Path;

use super::counters::OwnerSemanticVerificationCounters;
use super::outcome::{
    OwnerSemanticVerificationDenial, OwnerSemanticVerificationResourceDenial,
    OwnerSemanticVerificationResult,
};
use crate::backup_verification::owner_resource_budget::actual_owner_result_owned_allocation_bytes;
use crate::inspection::OwnerDecodedArtifactBinding;
use crate::truth_composition::RecoveryCandidateObservation;
use crate::OfflinePhysicalArtifactFamily;
use worth_store_physical_format::BackupBundleManifest;

pub(super) fn close(
    root: &Path,
    manifest: &BackupBundleManifest,
    max_buffer_bytes: usize,
    maximum_owned_allocation_bytes: u64,
    counters: OwnerSemanticVerificationCounters,
    recovery_candidates: Vec<RecoveryCandidateObservation>,
    mut owner_bindings: Vec<OwnerDecodedArtifactBinding>,
) -> Result<OwnerSemanticVerificationResult, OwnerSemanticVerificationDenial> {
    owner_bindings.push(
        OwnerDecodedArtifactBinding::new(
            root.join("backup.manifest"),
            OfflinePhysicalArtifactFamily::Manifest,
            manifest.manifest_generation(),
        )
        .expect("admitted manifest generation is nonzero"),
    );
    let actual_owned_allocation_bytes = actual_owner_result_owned_allocation_bytes(
        &recovery_candidates,
        &owner_bindings,
        max_buffer_bytes,
    )
    .ok_or(OwnerSemanticVerificationDenial::Resource(
        OwnerSemanticVerificationResourceDenial {
            required_bytes: u64::MAX,
            limit_bytes: maximum_owned_allocation_bytes,
        },
    ))?;
    if actual_owned_allocation_bytes > maximum_owned_allocation_bytes {
        return Err(OwnerSemanticVerificationDenial::Resource(
            OwnerSemanticVerificationResourceDenial {
                required_bytes: actual_owned_allocation_bytes,
                limit_bytes: maximum_owned_allocation_bytes,
            },
        ));
    }
    Ok(OwnerSemanticVerificationResult {
        counters,
        recovery_candidates,
        owner_bindings,
        peak_owned_allocation_bytes: actual_owned_allocation_bytes,
    })
}
