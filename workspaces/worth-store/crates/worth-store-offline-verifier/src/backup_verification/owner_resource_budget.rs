use std::path::Path;

use crate::backup_verification::verification_owned_memory::allocation_bytes;
use crate::inspection::OwnerDecodedArtifactBinding;
use crate::truth_composition::RecoveryCandidateObservation;
use worth_store_physical_format::BackupBundleManifest;

pub(super) fn maximum_requested_owned_allocation_bytes(
    root: &Path,
    manifest: &BackupBundleManifest,
) -> Option<u64> {
    maximum_reserved_owned_allocation_bytes(
        root,
        manifest,
        manifest.artifacts().len(),
        manifest.artifacts().len().checked_add(1)?,
        0,
    )
}

pub(super) fn maximum_reserved_owned_allocation_bytes(
    root: &Path,
    manifest: &BackupBundleManifest,
    recovery_candidate_capacity: usize,
    owner_binding_capacity: usize,
    decoder_buffer_bytes: usize,
) -> Option<u64> {
    let row_storage =
        allocation_bytes::<RecoveryCandidateObservation>(recovery_candidate_capacity)?
            .checked_add(allocation_bytes::<OwnerDecodedArtifactBinding>(
                owner_binding_capacity,
            )?)?
            .checked_add(u64::try_from(decoder_buffer_bytes).ok()?)?;
    manifest
        .artifacts()
        .iter()
        .try_fold(row_storage, |total, row| {
            let binding_path = joined_path_payload_bytes(root, row.output_name())?;
            let possible_defect_name = u64::try_from(row.output_name().len()).ok()?;
            total
                .checked_add(binding_path)?
                .checked_add(possible_defect_name)
        })?
        .checked_add(joined_path_payload_bytes(root, "backup.manifest")?)
}

pub(super) fn actual_owner_result_owned_allocation_bytes(
    recovery_candidates: &Vec<RecoveryCandidateObservation>,
    owner_bindings: &Vec<OwnerDecodedArtifactBinding>,
    decoder_buffer_bytes: usize,
) -> Option<u64> {
    let row_storage =
        allocation_bytes::<RecoveryCandidateObservation>(recovery_candidates.capacity())?
            .checked_add(allocation_bytes::<OwnerDecodedArtifactBinding>(
                owner_bindings.capacity(),
            )?)?
            .checked_add(u64::try_from(decoder_buffer_bytes).ok()?)?;
    owner_bindings
        .iter()
        .try_fold(row_storage, |total, binding| {
            total.checked_add(binding.owned_allocation_bytes()?)
        })
}

#[cfg(windows)]
fn joined_path_payload_bytes(root: &Path, output_name: &str) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    let root_units = u64::try_from(root.as_os_str().encode_wide().count()).ok()?;
    let name_units = u64::try_from(output_name.encode_utf16().count()).ok()?;
    root_units
        .checked_add(1)?
        .checked_add(name_units)?
        .checked_mul(2)
}

#[cfg(unix)]
fn joined_path_payload_bytes(root: &Path, output_name: &str) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    u64::try_from(root.as_os_str().as_bytes().len())
        .ok()?
        .checked_add(1)?
        .checked_add(u64::try_from(output_name.len()).ok()?)
}

#[cfg(not(any(windows, unix)))]
fn joined_path_payload_bytes(root: &Path, output_name: &str) -> Option<u64> {
    u64::try_from(root.to_string_lossy().len())
        .ok()?
        .checked_add(1)?
        .checked_add(u64::try_from(output_name.len()).ok()?)
}
