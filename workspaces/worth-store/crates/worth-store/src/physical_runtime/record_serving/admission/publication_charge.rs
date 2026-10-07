use worth_store_physical_format::{
    DurablePhysicalRootManifest, BOOTSTRAP_CATALOG_BYTES, ROOT_SELECTOR_BYTES,
};

use super::super::{
    admission::bootstrap::{BootstrapTransitionFailure, RecordBootstrapDenial},
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy,
};
use super::current_free_space::load_free_space_manifest_with_len;
use super::open::CurrentRootAdmission;

mod root_blocks;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct ReconstructedPublicationMetadata {
    pub(in crate::physical_runtime) generation: u64,
    pub(in crate::physical_runtime) record_count: u64,
    pub(in crate::physical_runtime) node_capacity: u16,
    pub(in crate::physical_runtime) bytes: u64,
}

/// Exact metadata emitted by each authenticated publication root, oldest first.
/// The WAL owner later selects the entries still represented by retained groups.
pub(super) fn publication_overheads(
    admission: &CurrentRootAdmission<'_>,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
    roots: &[DurablePhysicalRootManifest],
    root_lengths: &[u64],
) -> Result<Vec<ReconstructedPublicationMetadata>, BootstrapTransitionFailure> {
    if roots.len() != root_lengths.len() {
        return Err(damaged());
    }
    let mut overheads = Vec::new();
    overheads
        .try_reserve_exact(roots.len())
        .map_err(|_| metadata_pressure())?;
    for (root, root_length) in roots.iter().zip(root_lengths) {
        if root.generation() == 1 {
            overheads.push(ReconstructedPublicationMetadata {
                generation: 1,
                record_count: root.record_count(),
                node_capacity: root.node_capacity(),
                bytes: 0,
            });
            continue;
        }
        let historical = CurrentRootAdmission {
            generation: root.generation(),
            lifecycle: std::sync::Arc::clone(&admission.lifecycle),
            ..admission.clone()
        };
        let (free_space, free_space_length) = load_free_space_manifest_with_len(&historical, root)?;
        let routing =
            root_blocks::new_generation_bytes(&historical, format, access, root, &free_space)?;
        let fixed = root_length
            .checked_add(free_space_length)
            .and_then(|bytes| bytes.checked_add(2 * ROOT_SELECTOR_BYTES as u64))
            .and_then(|bytes| bytes.checked_add(BOOTSTRAP_CATALOG_BYTES as u64))
            .ok_or_else(damaged)?;
        let total = fixed.checked_add(routing).ok_or_else(damaged)?;
        overheads.push(ReconstructedPublicationMetadata {
            generation: root.generation(),
            record_count: root.record_count(),
            node_capacity: root.node_capacity(),
            bytes: total,
        });
    }
    Ok(overheads)
}

pub(super) fn damaged() -> BootstrapTransitionFailure {
    BootstrapTransitionFailure::Denied(RecordBootstrapDenial::CurrentRootDamaged)
}

pub(super) fn metadata_pressure() -> BootstrapTransitionFailure {
    BootstrapTransitionFailure::Denied(RecordBootstrapDenial::from_residency(
        worth_store_buffer_pool::PhysicalResidencyDenial::MetadataBudgetExceeded,
    ))
}
