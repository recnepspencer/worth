//! Read one selected control through its exact C.9 extent membership.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_extent_chunk, DurableExtentManifest, DurableExtentRecordPlacement,
    ExtentArenaFrameLayout, ExtentChunkCoordinate, PhysicalRecordFormatDeclaration,
    RecordArtifactFile, DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};
use worth_store_physical_integrity::{
    validate_extent_chunk_membership, validate_extent_manifest, ExtentChunkIntegrityValidation,
    ExtentManifestIntegrityValidation, PhysicalArtifactScope, PhysicalByteRange,
    SelectedExtentPayloadBuilder, UntrustedPhysicalArtifact,
};

use super::super::control_frames::SelectedArtifactSlice;
use super::super::SelectedMediaRejoinDenial as Denial;

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    placement: DurableExtentRecordPlacement,
    maximum: u64,
    slices: &mut Vec<SelectedArtifactSlice>,
) -> Result<Vec<u8>, Denial> {
    read_with_slice_limit(discovery, format, placement, maximum, slices, usize::MAX)
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn read_with_slice_limit(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    placement: DurableExtentRecordPlacement,
    maximum: u64,
    slices: &mut Vec<SelectedArtifactSlice>,
    max_slices: usize,
) -> Result<Vec<u8>, Denial> {
    if placement.payload_bytes() == 0 || placement.payload_bytes() > maximum {
        return Err(Denial::BoundExceeded);
    }
    let page_limit = u64::from(format.page_size().bytes());
    let observed = discovery
        .read_extent_manifest(placement.arena_range(), page_limit)
        .map_err(Denial::Discovery)?;
    let manifest_bytes = observed.bytes().ok_or(Denial::MissingFrame)?;
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
    // The integrity-validated manifest fixes the exact number of slices this
    // read will append. Deny and reserve before reading any chunk payload.
    let additional = usize::try_from(manifest.chunk_count())
        .ok()
        .and_then(|chunks| chunks.checked_add(1))
        .ok_or(Denial::BoundExceeded)?;
    if slices
        .len()
        .checked_add(additional)
        .filter(|count| *count <= max_slices)
        .is_none()
    {
        return Err(Denial::BoundExceeded);
    }
    slices
        .try_reserve_exact(additional)
        .map_err(|_| Denial::BoundExceeded)?;
    let artifact = RecordArtifactFile::ExtentArena {
        arena: placement.arena_range().arena().get(),
    };
    slices.push(
        SelectedArtifactSlice::observed(
            artifact,
            placement.arena_range().offset(),
            manifest_bytes,
            false,
        )
        .ok_or(Denial::ControlFrame)?,
    );
    let mut builder = SelectedExtentPayloadBuilder::new(validated.membership(), placement)
        .ok_or(Denial::ControlFrame)?;
    let mut payload = Vec::with_capacity(
        usize::try_from(placement.payload_bytes()).map_err(|_| Denial::BoundExceeded)?,
    );
    for ordinal in 1..=manifest.chunk_count() {
        let coordinate = ExtentChunkCoordinate::new(
            placement.record(),
            manifest.extent_cell(),
            manifest.logical_bytes(),
            payload.len() as u64,
            ordinal,
        )
        .ok_or(Denial::ControlFrame)?;
        let length = (manifest.logical_bytes() as usize - payload.len())
            .min(manifest.chunk_payload_capacity() as usize);
        let frame_length = DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES + length;
        let relative = layout.chunk_offset(ordinal).ok_or(Denial::ControlFrame)?;
        let frame = discovery
            .read_extent_range(
                placement.arena_range(),
                relative,
                u32::try_from(frame_length).map_err(|_| Denial::BoundExceeded)?,
                page_limit,
            )
            .map_err(Denial::Discovery)?;
        let bytes = frame.bytes().ok_or(Denial::MissingFrame)?;
        let absolute = placement
            .arena_range()
            .offset()
            .checked_add(relative)
            .ok_or(Denial::ControlFrame)?;
        let scope = PhysicalArtifactScope::extent_chunk(
            discovery.store_identity(),
            format,
            coordinate,
            PhysicalByteRange::new(absolute, frame_length as u64)
                .map_err(|_| Denial::ControlFrame)?,
            placement.arena_range(),
        );
        let input = UntrustedPhysicalArtifact::from_bounded_bytes(bytes);
        let (validated_chunk, _) =
            validate_extent_chunk_membership(input, scope, validated.membership());
        let ExtentChunkIntegrityValidation::Intact(chunk) = validated_chunk else {
            return Err(Denial::ControlFrame);
        };
        slices.push(
            SelectedArtifactSlice::observed(artifact, absolute, bytes, false)
                .ok_or(Denial::ControlFrame)?,
        );
        builder.append(&chunk, input).ok_or(Denial::ControlFrame)?;
        let (chunk_payload, chunk_format) =
            decode_extent_chunk(bytes, coordinate).map_err(|_| Denial::ControlFrame)?;
        if chunk_format != format || chunk_payload.len() != length {
            return Err(Denial::ControlFrame);
        }
        payload.extend_from_slice(chunk_payload);
    }
    let witness = builder.finish().ok_or(Denial::ControlFrame)?;
    if !witness.matches_frame(&payload) {
        return Err(Denial::ControlFrame);
    }
    Ok(payload)
}

#[cfg(test)]
#[path = "no_release_frame/tests.rs"]
mod tests;
