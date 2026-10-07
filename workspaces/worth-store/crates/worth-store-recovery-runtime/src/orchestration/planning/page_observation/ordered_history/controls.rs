//! Exact WAL descriptor plus candidate-root V3 control triple.

use crate::orchestration::recovery_budget::RecoveryAllowance;
use sha2::{Digest, Sha256};
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV3, BlobRecordKind, BlobRecordV1,
    CurrentPhysicalRecordPlacement, DropSetManifestV3, PhysicalRecordFormatDeclaration,
    SelectedRecordContentClass,
};
use worth_store_recovery_physics::{
    AdmittedPhysicalRedoMembers, ReleasedInventoryView, VerifiedAddressedReleasedControlFrame,
    VerifiedReleasedRootEdge, WitnessedSelectedControlFrame,
};

use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;

use super::super::historical_drop::selected_control;
use super::walk_failure::{Verdict, WalkFailure, WalkFailure::Unverified};

pub(super) struct ReleasedControls {
    pub(super) descriptor: BlobReclaimDescriptorV3,
    pub(super) manifest: DropSetManifestV3,
    pub(super) descriptor_frame: WitnessedSelectedControlFrame,
    pub(super) reservation_frame: WitnessedSelectedControlFrame,
    pub(super) manifest_frame: WitnessedSelectedControlFrame,
}

pub(super) struct AddressedControls {
    pub(super) descriptor_frame: VerifiedAddressedReleasedControlFrame,
    pub(super) reservation_frame: VerifiedAddressedReleasedControlFrame,
    pub(super) manifest_frame: VerifiedAddressedReleasedControlFrame,
    pub(super) retained_bytes: u64,
}

pub(super) fn bind_addressed(
    edge: &VerifiedReleasedRootEdge,
    result: ReleasedInventoryView<'_>,
    controls: &ReleasedControls,
    format: PhysicalRecordFormatDeclaration,
    maximum_entries: u64,
    available_bytes: u64,
    staging: RecoveryAllowance,
) -> Result<AddressedControls, WalkFailure> {
    let refused = |denial| WalkFailure::control_refused(denial, staging);
    let descriptor_frame = VerifiedAddressedReleasedControlFrame::admit(
        edge,
        result,
        &controls.descriptor_frame,
        BlobRecordKind::ReclaimDescriptorV3,
        format,
        maximum_entries,
        available_bytes,
    )
    .map_err(refused)?;
    let remaining = available_bytes
        .checked_sub(descriptor_frame.retained_bytes())
        .proven()?;
    let reservation_frame = VerifiedAddressedReleasedControlFrame::admit(
        edge,
        result,
        &controls.reservation_frame,
        BlobRecordKind::OriginalDropReserved,
        format,
        maximum_entries,
        remaining,
    )
    .map_err(refused)?;
    let remaining = remaining
        .checked_sub(reservation_frame.retained_bytes())
        .proven()?;
    let manifest_frame = VerifiedAddressedReleasedControlFrame::admit(
        edge,
        result,
        &controls.manifest_frame,
        BlobRecordKind::DropSetManifestV3,
        format,
        maximum_entries,
        remaining,
    )
    .map_err(refused)?;
    let retained_bytes = descriptor_frame
        .retained_bytes()
        .checked_add(reservation_frame.retained_bytes())
        .proven()?
        .checked_add(manifest_frame.retained_bytes())
        .proven()?;
    Ok(AddressedControls {
        descriptor_frame,
        reservation_frame,
        manifest_frame,
        retained_bytes,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn released_controls(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &[CurrentPhysicalRecordPlacement],
    redo: &AdmittedPhysicalRedoMembers,
    operation: [u8; 32],
    descriptor_record: worth_store_physical_format::PersistedRecordIdentity,
    descriptor_sha256: [u8; 32],
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    resident: &mut ResidentAllowance,
    staging: RecoveryAllowance,
) -> Result<ReleasedControls, WalkFailure> {
    let mut members = redo
        .admitted_drop_members()
        .filter(|(id, _, _, _)| *id == operation);
    let (_, _, _, wal_record) = members.next().proven()?;
    if members.next().is_some() || <[u8; 32]>::from(Sha256::digest(wal_record)) != descriptor_sha256
    {
        return Err(Unverified);
    }
    let BlobRecordV1::ReclaimDescriptorV3(descriptor) = decode_blob_record(wal_record).proven()?
    else {
        return Err(Unverified);
    };
    let descriptor_frame = selected_control(
        discovery,
        routes,
        descriptor_record,
        BlobRecordKind::ReclaimDescriptorV3,
        format,
        budget,
        trace,
        resident,
        staging,
    )?;
    if descriptor_frame.bytes() != wal_record {
        return Err(Unverified);
    }
    let manifest_frame = selected_control(
        discovery,
        routes,
        descriptor.base().manifest_record(),
        BlobRecordKind::DropSetManifestV3,
        format,
        budget,
        trace,
        resident,
        staging,
    )?;
    let BlobRecordV1::DropSetManifestV3(manifest) =
        decode_blob_record(manifest_frame.bytes()).proven()?
    else {
        return Err(Unverified);
    };
    if <[u8; 32]>::from(Sha256::digest(manifest_frame.bytes()))
        != descriptor.base().manifest_frame_sha256()
        || manifest.store() != descriptor.base().store()
        || manifest.source_basis_digest() != descriptor.base().source_basis_digest()
        || manifest.count() != descriptor.base().manifest_count()
    {
        return Err(Unverified);
    }
    let mut reservation_frame = None;
    for route in routes.iter().filter(|route| {
        route.content_class()
            == SelectedRecordContentClass::Blob(BlobRecordKind::OriginalDropReserved)
    }) {
        let frame = selected_control(
            discovery,
            routes,
            route.record(),
            BlobRecordKind::OriginalDropReserved,
            format,
            budget,
            trace,
            resident,
            staging,
        )?;
        let BlobRecordV1::OriginalDropReserved(reservation) =
            decode_blob_record(frame.bytes()).proven()?
        else {
            return Err(Unverified);
        };
        if reservation.manifest_record() == descriptor.base().manifest_record()
            && reservation.request() == descriptor.custody().request()
        {
            if reservation_frame.replace(frame).is_some() {
                return Err(Unverified);
            }
        }
    }
    Ok(ReleasedControls {
        descriptor,
        manifest,
        descriptor_frame,
        reservation_frame: reservation_frame.proven()?,
        manifest_frame,
    })
}
