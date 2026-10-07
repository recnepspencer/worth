//! Join a WAL directory replacement to the actual selected source directory bytes.

use super::ResolvedPlanningBasis;
use crate::entry::PhysicalRecoveryPlanningDenial as Denial;
use crate::orchestration::planning::completion::historical_publication::HistoricalFailure;
use crate::orchestration::planning::{
    released_directory, selected_source_inventory::ResidentAllowance,
};
use worth_store_recovery_physics::{PhysicalRedoProjection, VerifiedReleasedDirectoryReplacement};

pub(super) fn admit(
    selection: &worth_store_recovery_physics::PhysicalSourceSelection,
    basis: &mut ResolvedPlanningBasis,
    projection_index: usize,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    discovery: &mut worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery,
    resident: &mut ResidentAllowance,
) -> Result<Option<VerifiedReleasedDirectoryReplacement>, Denial> {
    let projection: &PhysicalRedoProjection = &basis.redo.projections()[projection_index];
    let worth_store_physical_format::PersistedPhysicalRecoveryOperation::RecordsDropped {
        directory_replacement,
        ..
    } = projection.materialization().operation()
    else {
        unreachable!("pending released member");
    };
    let root = selection.root().selected().manifest();
    // Necessity is decided once, by the released transition's root semantics.
    let Some(replacement) = directory_replacement else {
        return Ok(None);
    };
    let record = replacement.expected_previous().directory_record();
    let route = selection
        .page_facts()
        .placements()
        .iter()
        .find(|route| route.record() == record)
        .copied()
        .ok_or(Denial::ReleasedDirectoryProof(
            worth_store_recovery_physics::ReleasedDirectoryReplacementDenial::SourceBinding,
        ))?;
    let bytes = released_directory::read(
        discovery,
        route,
        &basis.observed_pages.selected_source,
        format,
        &mut basis.observed_pages.manifest_budget,
        trace,
        resident,
    )
    .map_err(Denial::ReleasedDirectorySource)?;
    resident
        .transient(VerifiedReleasedDirectoryReplacement::maximum_decode_heap_bytes())
        .map_err(|_| {
            Denial::ReleasedDirectorySource(
                crate::entry::PhysicalRecoverySelectedRecordReadDenial::ResidentBoundExceeded,
            )
        })?;
    let proof = VerifiedReleasedDirectoryReplacement::admit_projection(
        projection,
        &basis.redo,
        root,
        route,
        &bytes,
        &basis.verified_drops,
        format,
    )
    .map_err(Denial::ReleasedDirectoryProof);
    let payload_backing = bytes.capacity() as u64;
    drop(bytes);
    // Handing back more than was held is no bound of the allowance.
    resident.release(payload_backing).map_err(|_| {
        Denial::ReleasedDirectorySource(
            crate::entry::PhysicalRecoverySelectedRecordReadDenial::InvalidPayload,
        )
    })?;
    proof.map(Some)
}

/// What a directory that could not be read says about recovery's limits.
pub(super) fn unread(denial: &Denial) -> Option<HistoricalFailure> {
    match denial {
        Denial::ReleasedDirectorySource(denial) => Some(HistoricalFailure::from(denial.clone())),
        _ => None,
    }
}
