//! C.9 extent-frame reading for a selected reclaim control.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_extent_chunk, DurableExtentManifest, DurableExtentRecordPlacement,
    ExtentArenaFrameLayout, ExtentChunkFrame, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};
use worth_store_physical_integrity::{
    validate_extent_chunk_membership, validate_extent_manifest, ExtentChunkIntegrityValidation,
    ExtentManifestIntegrityValidation, IntegrityValidatedSelectedExtentPayload,
    PhysicalArtifactScope, PhysicalByteRange, SelectedExtentPayloadBuilder,
    UntrustedPhysicalArtifact,
};

use super::{super::SelectedMediaRejoinDenial as Denial, SelectedArtifactSlice};
use crate::physical_runtime::recovery_construction::selected_rejoin::resident::StoreRejoinResidentLedger;
#[path = "extent_read/storage.rs"]
mod storage;
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) use storage::ExtentReadStorage;

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn read_extent(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    placement: DurableExtentRecordPlacement,
    slices: &mut Vec<SelectedArtifactSlice>,
) -> Result<(Vec<u8>, IntegrityValidatedSelectedExtentPayload), Denial> {
    read_extent_inner(discovery, format, placement, slices, &mut ())
}

/// `slices` is empty or its existing backing is already charged to `resident`.
/// The returned payload remains charged until its owner drops it and releases
/// its actual capacity from the same ledger.
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn read_extent_with_resident(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    placement: DurableExtentRecordPlacement,
    slices: &mut Vec<SelectedArtifactSlice>,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<(Vec<u8>, IntegrityValidatedSelectedExtentPayload), Denial> {
    read_extent_inner(discovery, format, placement, slices, resident)
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn read_extent_with_storage<
    S: ExtentReadStorage,
>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    placement: DurableExtentRecordPlacement,
    slices: &mut Vec<SelectedArtifactSlice>,
    storage: &mut S,
) -> Result<(Vec<u8>, IntegrityValidatedSelectedExtentPayload), Denial> {
    read_extent_inner(discovery, format, placement, slices, storage)
}

fn read_extent_inner<S: ExtentReadStorage>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    placement: DurableExtentRecordPlacement,
    slices: &mut Vec<SelectedArtifactSlice>,
    storage: &mut S,
) -> Result<(Vec<u8>, IntegrityValidatedSelectedExtentPayload), Denial> {
    let observed = storage.read_manifest(discovery, placement)?;
    let manifest_bytes = observed.bytes().ok_or(Denial::MissingFrame)?;
    let arena_artifact = RecordArtifactFile::ExtentArena {
        arena: placement.arena_range().arena().get(),
    };
    storage.grow_slices(slices)?;
    slices.push(
        SelectedArtifactSlice::observed(
            arena_artifact,
            placement.arena_range().offset(),
            manifest_bytes,
            false,
        )
        .ok_or(Denial::BoundExceeded)?,
    );
    let scope = PhysicalArtifactScope::extent_manifest(
        discovery.store_identity(),
        format,
        placement,
        PhysicalByteRange::new(
            placement.arena_range().offset(),
            manifest_bytes.len() as u64,
        )
        .map_err(|_| Denial::ControlFrame)?,
    );
    let (validated, _) = validate_extent_manifest(
        UntrustedPhysicalArtifact::from_bounded_bytes(manifest_bytes),
        scope,
    );
    let ExtentManifestIntegrityValidation::Intact(validated) = validated else {
        return Err(Denial::ControlFrame);
    };
    let (manifest, observed_format) =
        DurableExtentManifest::decode(manifest_bytes).map_err(|_| Denial::ControlFrame)?;
    if observed_format != format
        || manifest.record() != placement.record()
        || manifest.extent_cell() != placement.extent_cell()
        || manifest.logical_bytes() != placement.payload_bytes()
    {
        return Err(Denial::ControlFrame);
    }
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment())
        .filter(|layout| layout.admits(placement.arena_range(), manifest.chunk_count()))
        .ok_or(Denial::ControlFrame)?;
    let mut builder = SelectedExtentPayloadBuilder::new(validated.membership(), placement)
        .ok_or(Denial::ControlFrame)?;
    let payload_length =
        usize::try_from(placement.payload_bytes()).map_err(|_| Denial::BoundExceeded)?;
    let mut payload = storage.reserve_payload(payload_length)?;
    for ordinal in 1..=manifest.chunk_count() {
        let framed = ExtentChunkFrame::of(manifest, layout, ordinal).ok_or(Denial::ControlFrame)?;
        let coordinate = framed.coordinate();
        let relative = framed.offset();
        let frame = storage.read_chunk(discovery, placement, relative, framed.length())?;
        let bytes = frame.bytes().ok_or(Denial::MissingFrame)?;
        let absolute = placement
            .arena_range()
            .offset()
            .checked_add(relative)
            .ok_or(Denial::ControlFrame)?;
        storage.grow_slices(slices)?;
        slices.push(
            SelectedArtifactSlice::observed(arena_artifact, absolute, bytes, false)
                .ok_or(Denial::BoundExceeded)?,
        );
        let scope = PhysicalArtifactScope::extent_chunk(
            discovery.store_identity(),
            format,
            coordinate,
            PhysicalByteRange::new(absolute, u64::from(framed.length()))
                .map_err(|_| Denial::ControlFrame)?,
            placement.arena_range(),
        );
        let input = UntrustedPhysicalArtifact::from_bounded_bytes(bytes);
        let (validated_chunk, _) =
            validate_extent_chunk_membership(input, scope, validated.membership());
        let ExtentChunkIntegrityValidation::Intact(chunk) = validated_chunk else {
            return Err(Denial::ControlFrame);
        };
        builder.append(&chunk, input).ok_or(Denial::ControlFrame)?;
        let (chunk_payload, chunk_format) =
            decode_extent_chunk(bytes, coordinate).map_err(|_| Denial::ControlFrame)?;
        if chunk_format != format || chunk_payload.len() != framed.payload_bytes() as usize {
            return Err(Denial::ControlFrame);
        }
        payload.extend_from_slice(chunk_payload);
        storage.discard_frame(frame)?;
    }
    let witness = builder.finish().ok_or(Denial::ControlFrame)?;
    if !witness.matches_frame(&payload) {
        return Err(Denial::ControlFrame);
    }
    storage.discard_frame(observed)?;
    Ok((payload, witness))
}
