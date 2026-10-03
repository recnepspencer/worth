//! Independent selected-media reread of a released member's old and new
//! directory frames. The old route may be inline; the new member is extent.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    RecordArtifactFile, RecordSegmentPageManifestEntry, SelectedRecordContentClass,
    MAX_DERIVED_FAMILY_ROOTS,
};
use worth_store_physical_integrity::{
    validate_inline_page, InlinePageIntegrityValidation, PhysicalArtifactScope, PhysicalByteRange,
    UntrustedPhysicalArtifact,
};
use worth_store_recovery_physics::{
    VerifiedReleasedDirectoryReplacement, VerifiedReleasedV3InventoryTransition,
};

use super::super::{
    control_frames::{
        read_extent_with_storage, ExtentReadStorage, SelectedArtifactSlice,
        SelectedControlMediaFingerprint,
    },
    tier::routes::RouteWalkStorage,
    SelectedMediaRejoinDenial as Denial,
};
use super::delta::Snapshot;

const MAX_DIRECTORY_FRAME_BYTES: u64 = (101 + MAX_DERIVED_FAMILY_ROOTS * 26) as u64;

/// Rereads the replacement directory frames of one admitted V3 edge, when the
/// edge carries a replacement; `None` when the watermark was not invalidated.
#[allow(clippy::too_many_arguments)]
pub(super) fn verify_edge_with_storage<S: RouteWalkStorage + ExtentReadStorage>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    source_root: &DurablePhysicalRootManifest,
    source: &Snapshot,
    result_root: &DurablePhysicalRootManifest,
    result: &Snapshot,
    transition: &VerifiedReleasedV3InventoryTransition,
    format: PhysicalRecordFormatDeclaration,
    storage: &mut S,
) -> Result<Option<SelectedControlMediaFingerprint>, Denial> {
    transition
        .directory_replacement()
        .map(|proof| {
            verify_with_storage(
                discovery,
                source_root,
                source,
                result_root,
                result,
                proof,
                format,
                storage,
            )
        })
        .transpose()
}

#[allow(clippy::too_many_arguments)]
fn verify_with_storage<S: RouteWalkStorage + ExtentReadStorage>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    source_root: &DurablePhysicalRootManifest,
    source: &Snapshot,
    result_root: &DurablePhysicalRootManifest,
    result: &Snapshot,
    proof: &VerifiedReleasedDirectoryReplacement,
    format: PhysicalRecordFormatDeclaration,
    storage: &mut S,
) -> Result<SelectedControlMediaFingerprint, Denial> {
    let source_record = proof.source_binding().directory_record();
    let result_record = proof.result_binding().directory_record();
    let source_route = route(&source.routes, source_record)?;
    let result_route = route(&result.routes, result_record)?;
    if source_route.content_class() != SelectedRecordContentClass::DerivedDirectory
        || result_route.content_class() != SelectedRecordContentClass::DerivedDirectory
    {
        return Err(Denial::RoutingFrame);
    }
    let mut slices = storage.reserve_vec::<SelectedArtifactSlice>(0)?;
    let source_bytes = read_selected(
        discovery,
        source_route,
        &source.segments,
        format,
        &mut slices,
        storage,
    )?;
    let result_bytes = read_selected(
        discovery,
        result_route,
        &result.segments,
        format,
        &mut slices,
        storage,
    )?;
    let checked = proof.verify_selected_media(
        source_root,
        source_route,
        &source_bytes,
        result_root,
        result_route,
        &result_bytes,
    );
    storage.discard_vec(source_bytes)?;
    storage.discard_vec(result_bytes)?;
    checked.map_err(|_| Denial::RoutingFrame)?;
    Ok(SelectedControlMediaFingerprint::observed(slices))
}

fn route(
    routes: &[CurrentPhysicalRecordPlacement],
    record: worth_store_physical_format::PersistedRecordIdentity,
) -> Result<CurrentPhysicalRecordPlacement, Denial> {
    routes
        .binary_search_by_key(&record, |route| route.record())
        .ok()
        .map(|index| routes[index])
        .ok_or(Denial::MissingRoute)
}

fn read_selected<S: RouteWalkStorage + ExtentReadStorage>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    route: CurrentPhysicalRecordPlacement,
    segments: &[RecordSegmentPageManifestEntry],
    format: PhysicalRecordFormatDeclaration,
    slices: &mut Vec<SelectedArtifactSlice>,
    storage: &mut S,
) -> Result<Vec<u8>, Denial> {
    if route.payload_bytes() == 0 || route.payload_bytes() > MAX_DIRECTORY_FRAME_BYTES {
        return Err(Denial::BoundExceeded);
    }
    match route {
        CurrentPhysicalRecordPlacement::Extent(extent) => {
            read_extent_with_storage(discovery, format, extent, slices, storage)
                .map(|(bytes, _)| bytes)
        }
        CurrentPhysicalRecordPlacement::Inline(inline) => {
            let page = segments
                .iter()
                .find(|entry| {
                    entry.page_cell() == inline.page_cell()
                        && entry.data_segment_cell() == inline.segment_cell()
                        && entry.data_page_count() <= inline.segment_page_capacity()
                })
                .ok_or(Denial::RoutingFrame)?;
            let length = format.page_size().bytes();
            let offset = u64::from(page.frame_index())
                .checked_mul(u64::from(length))
                .ok_or(Denial::BoundExceeded)?;
            let artifact = RecordArtifactFile::Segment {
                segment: inline.segment().get(),
                generation: inline.segment_generation(),
            };
            let frame = storage.read_page_range(discovery, artifact, offset, length)?;
            let bytes = frame.bytes().ok_or(Denial::MissingFrame)?;
            let range = PhysicalByteRange::new(offset, bytes.len() as u64)
                .map_err(|_| Denial::RoutingFrame)?;
            let scope = PhysicalArtifactScope::inline_page(
                discovery.store_identity(),
                format,
                inline.page_cell(),
                range,
            );
            let (validation, _) =
                validate_inline_page(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
            let InlinePageIntegrityValidation::Intact(validated) = validation else {
                return Err(Denial::RoutingFrame);
            };
            let payload = validated
                .project_record(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), inline)
                .map_err(|_| Denial::RoutingFrame)?
                .payload_range();
            if payload.len() as u64 > MAX_DIRECTORY_FRAME_BYTES {
                return Err(Denial::BoundExceeded);
            }
            let mut selected = storage.reserve_vec(payload.len())?;
            selected.extend_from_slice(&bytes[payload]);
            storage.grow_vec(slices, 1)?;
            slices.push(
                SelectedArtifactSlice::observed(artifact, offset, bytes, true)
                    .ok_or(Denial::BoundExceeded)?,
            );
            RouteWalkStorage::discard_frame(storage, frame)?;
            Ok(selected)
        }
    }
}
