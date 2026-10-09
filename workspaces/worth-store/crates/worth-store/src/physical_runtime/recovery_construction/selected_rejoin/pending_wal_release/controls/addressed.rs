//! Checkpoint-source Batch controls are addressed below the final selected
//! root. Read the entire rooted control catalog; a tip-only match is not a
//! substitute for the checkpoint's Store-wide Batch roster.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
    DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};
use worth_store_recovery_physics::{
    VerifiedAddressedCheckpointReleaseBase, VerifiedPendingWalReleaseCustody,
};

use super::super::super::{
    control_frames::{self, ObservedSelectedControls, SelectedArtifactSlice},
    tier, SelectedMediaRejoinDenial as Denial, MAX_CONTROL_FRAME_BYTES, MAX_DISCOVERY_BYTES,
};
use super::super::{addressed_root, selection::Selection};

pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selected: &Selection,
    format: PhysicalRecordFormatDeclaration,
    base: &VerifiedAddressedCheckpointReleaseBase,
    claim: &VerifiedPendingWalReleaseCustody,
    retained_at_source: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
) -> Result<
    (
        ObservedSelectedControls,
        tier::routes::NoReleaseControlProvenance,
        Vec<SelectedArtifactSlice>,
    ),
    Denial,
> {
    let history = claim.ordered_history().ok_or(Denial::CertificateRoster)?;
    let first_topology = history
        .edges()
        .first()
        .ok_or(Denial::CertificateRoster)?
        .source_topology();
    let checkpoint = addressed_root::observe(
        discovery,
        base.checkpoint_root().generation(),
        selected.root.node_capacity(),
        format,
        first_topology,
        Some(base.checkpoint_root_frame_sha256()),
    )?;
    let free_sha: [u8; 32] = Sha256::digest(checkpoint.free.encode(format)).into();
    if checkpoint.root != *base.checkpoint_root()
        || free_sha != base.checkpoint_free_space_frame_sha256()
    {
        return Err(Denial::CheckpointBinding);
    }
    let routes = tier::routes::verify(discovery, &checkpoint.root, &checkpoint.free, format, None)?;
    let max_slices = preflight(base, routes.selected_routes(), &checkpoint.root, format)?;
    let partition = tier::released_partition::partition_with_slice_limit(
        discovery,
        routes.selected_routes(),
        format,
        max_slices,
    )?;
    if retained_at_source
        .keys()
        .any(|record| !partition.released().contains_key(record))
    {
        return Err(Denial::CertificateRoster);
    }
    let mut slices = Vec::new();
    slices
        .try_reserve_exact(partition.reservation_slices().len())
        .map_err(|_| Denial::BoundExceeded)?;
    slices.extend_from_slice(partition.reservation_slices());
    tier::verify_failed_ingest_subset_bounded(
        discovery,
        partition.failed_ingest(),
        format,
        checkpoint.root.generation(),
        base.checkpoint().source().identity().sequence().get(),
        &mut slices,
        max_slices,
    )?;
    let observed = control_frames::observe_addressed_base(
        discovery,
        &checkpoint.root,
        partition.released(),
        format,
        base,
        checkpoint.free.tier_epoch_start(),
        MAX_CONTROL_FRAME_BYTES,
    )?;
    Ok((observed, routes, slices))
}

fn preflight(
    base: &VerifiedAddressedCheckpointReleaseBase,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    root: &worth_store_physical_format::DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
) -> Result<usize, Denial> {
    let route_charge = root
        .record_count()
        .checked_mul(
            4 * (std::mem::size_of::<PersistedRecordIdentity>()
                + std::mem::size_of::<CurrentPhysicalRecordPlacement>()
                + 64) as u64,
        )
        .ok_or(Denial::BoundExceeded)?;
    let payload_charge = routes.values().try_fold(0_u64, |sum, route| {
        let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
            return Ok(sum);
        };
        sum.checked_add(
            extent
                .payload_bytes()
                .checked_mul(4)
                .ok_or(Denial::BoundExceeded)?,
        )
        .ok_or(Denial::BoundExceeded)
    })?;
    let fixed = route_charge
        .checked_add(payload_charge)
        .and_then(|value| value.checked_add(base.retained_bytes()))
        .and_then(|value| value.checked_add(4 * MAX_CONTROL_FRAME_BYTES))
        .ok_or(Denial::BoundExceeded)?;
    let remaining = MAX_DISCOVERY_BYTES
        .checked_sub(fixed)
        .ok_or(Denial::BoundExceeded)?;
    let slice_width = (std::mem::size_of::<SelectedArtifactSlice>() as u64)
        .checked_mul(8)
        .ok_or(Denial::BoundExceeded)?;
    let max_slices = usize::try_from(remaining / slice_width).map_err(|_| Denial::BoundExceeded)?;
    let capacity = u64::from(format.page_size().bytes())
        .checked_sub((DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES) as u64)
        .ok_or(Denial::BoundExceeded)?;
    let needed = routes.values().try_fold(0_u64, |sum, route| {
        let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
            return Ok(sum);
        };
        sum.checked_add(extent.payload_bytes().div_ceil(capacity))
            .and_then(|value| value.checked_add(1))
            .ok_or(Denial::BoundExceeded)
    })?;
    (needed <= max_slices as u64)
        .then_some(max_slices)
        .ok_or(Denial::BoundExceeded)
}
