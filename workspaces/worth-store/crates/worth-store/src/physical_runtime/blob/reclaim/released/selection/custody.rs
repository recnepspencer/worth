use worth_store_physical_format::{
    BlobReclaimDescriptorV2, BlobReclaimSourceBasisV1, OriginalDropReservedV1,
    PersistedRecordIdentity, ReleaseCustodyHeadKeyV1, ReleasedDropCustodyV1,
    ReleasedGenerationReclaimBasisV1,
};

use crate::physical_runtime::{
    BlobPhysicalAllocation, PhysicalRecordReader, ServingPhysicalRuntime,
};

use super::super::super::{scan, BlobReclaimFailure, BlobReclaimLimits};
use super::{chain, external_edges, graph, inventory::SelectedReleaseInventory, plan, resources};

/// The only Store custody eligible for V3: it was re-derived under the exact
/// protected source root selected *after* the reservation publication.
pub(in crate::physical_runtime::blob::reclaim) struct CertifiedPendingReleasedDrop {
    _reader: PhysicalRecordReader,
    source_root_frame_sha256: [u8; 32],
    source_free_space_frame_sha256: [u8; 32],
    transcripts: [[u8; 32]; 4],
}

impl CertifiedPendingReleasedDrop {
    pub(in crate::physical_runtime::blob::reclaim) fn custody(
        &self,
        request: worth_store_physical_format::OriginalDropReservationRequestV1,
    ) -> Result<ReleasedDropCustodyV1, BlobReclaimFailure> {
        ReleasedDropCustodyV1::new(
            self.source_root_frame_sha256,
            self.source_free_space_frame_sha256,
            self.transcripts[0],
            self.transcripts[1],
            self.transcripts[2],
            self.transcripts[3],
            request,
        )
        .map_err(BlobReclaimFailure::Format)
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::physical_runtime::blob::reclaim) fn certify(
    runtime: &ServingPhysicalRuntime,
    reader: PhysicalRecordReader,
    source_root_frame_sha256: [u8; 32],
    source_free_space_frame_sha256: [u8; 32],
    basis: ReleasedGenerationReclaimBasisV1,
    descriptor: BlobReclaimDescriptorV2,
    manifest_record: PersistedRecordIdentity,
    manifest_sha256: [u8; 32],
    reservation_record: PersistedRecordIdentity,
    reservation_sha256: [u8; 32],
    expected_reservation: OriginalDropReservedV1,
    expected_dropped: &[PersistedRecordIdentity],
    limits: BlobReclaimLimits,
    allocation: &BlobPhysicalAllocation<'_>,
) -> Result<CertifiedPendingReleasedDrop, BlobReclaimFailure> {
    resources::require_grant(limits, allocation)?;
    if reader.protected_root().root().generation().get() != descriptor.source_root_generation()
        || descriptor.store() != basis.publication().store()
        || descriptor.source_basis_digest()
            != BlobReclaimSourceBasisV1::ReleasedGeneration(basis).digest(descriptor.store())
        || descriptor.manifest_record() != manifest_record
        || descriptor.manifest_frame_sha256() != manifest_sha256
        || descriptor.manifest_count() as usize != expected_dropped.len()
        || expected_reservation.reclaim_attempt() != descriptor.reclaim_attempt()
        || expected_reservation.manifest_record() != manifest_record
        || expected_reservation.manifest_frame_sha256() != manifest_sha256
        || expected_reservation.source_basis_digest() != descriptor.source_basis_digest()
        || expected_reservation.reserved_selected_generation()
            != descriptor.source_root_generation()
    {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let mut scratch = resources::scratch()?;
    let mut work = scan::ReclaimInspectionWork::default();
    let (reader, mut inventory) = SelectedReleaseInventory::scan(
        reader.into_rebuild(),
        basis,
        limits,
        &mut scratch,
        &mut work,
    )?;
    let current_manifest = inventory
        .manifests
        .iter()
        .position(|link| link.record == manifest_record)
        .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?;
    let current_manifest = inventory.manifests.remove(current_manifest);
    if current_manifest.frame_sha256 != manifest_sha256
        || current_manifest.reclaim_attempt != descriptor.reclaim_attempt()
        || current_manifest.count != descriptor.manifest_count()
        || current_manifest.manifest_selected_generation
            != expected_reservation.manifest_selected_generation()
    {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let current_reservation = inventory
        .reservations
        .iter()
        .position(|link| link.record == reservation_record)
        .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?;
    let current_reservation = inventory.reservations.remove(current_reservation);
    if current_reservation.frame_sha256 != reservation_sha256
        || current_reservation.reservation != expected_reservation
        || inventory
            .manifests
            .iter()
            .any(|link| link.reclaim_attempt == descriptor.reclaim_attempt())
        || inventory
            .reservations
            .iter()
            .any(|link| link.reservation.reclaim_attempt() == descriptor.reclaim_attempt())
        || inventory
            .descriptors
            .iter()
            .any(|link| link.descriptor.reclaim_attempt() == descriptor.reclaim_attempt())
    {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    // Only completed predecessor descriptors enter this validation. The
    // selected current manifest/reservation are authenticated above but have
    // no descriptor or durable drop fate yet.
    let key = ReleaseCustodyHeadKeyV1::new(basis.object(), basis.generation())
        .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?;
    let selected_head = runtime
        .selected_release_head_for_root(reader.protected_root(), key)
        .map_err(|cause| BlobReclaimFailure::ReleaseCustodyCertification {
            manifest_record,
            reservation_record,
            cause,
        })?;
    let prior = chain::validate(
        &reader,
        &inventory,
        selected_head,
        limits,
        &mut scratch,
        &mut work,
    )?;
    if prior.predecessor != descriptor.predecessor()
        || prior.terminal
        || prior
            .cumulative_dropped
            .checked_add(expected_dropped.len() as u64)
            != Some(descriptor.cumulative_dropped())
    {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let closure = graph::authenticate_selected_closure(
        &reader,
        &mut inventory,
        &prior,
        limits,
        &mut scratch,
        &mut work,
    )?;
    let (reader, publication_referenced, external) =
        external_edges::protect(reader, &mut inventory, limits, &mut scratch, &mut work)?;
    graph::propagate_protected_subtrees(&reader, &mut inventory, limits, &mut scratch, &mut work)?;
    let planned = plan::select_post_order(
        &reader,
        &inventory,
        &prior,
        publication_referenced,
        limits,
        &mut scratch,
        &mut work,
    )?;
    if planned.dropped != expected_dropped
        || planned.cumulative_dropped != descriptor.cumulative_dropped()
        || planned.terminal != descriptor.terminal()
    {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    Ok(CertifiedPendingReleasedDrop {
        _reader: reader,
        source_root_frame_sha256,
        source_free_space_frame_sha256,
        transcripts: [
            inventory.selected_route_inventory_sha256,
            closure,
            external,
            planned.postorder_drop_sha256,
        ],
    })
}
