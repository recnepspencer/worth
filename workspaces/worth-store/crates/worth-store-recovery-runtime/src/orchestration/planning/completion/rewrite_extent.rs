use sha2::{Digest, Sha256};
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_extent_chunk, encode_data_frame_page_lsn, prepare_extent_chunk,
    CurrentPhysicalRecordPlacement, DurableExtentManifest, DurableExtentRecordPlacement,
    DurableFrameKind, ExtentChunkCoordinate, PersistedPhysicalDataFrameSubject,
    PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryManifest,
    PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageLsn, PhysicalRecordFormatDeclaration,
    PhysicalRewriteRedo, RecordArtifactFile, RecordFrameCoordinate,
};
use worth_store_recovery_physics::{
    PhysicalRedoProjection, PhysicalRewriteAdmission, PhysicalSourceSelection,
};

use super::decode_record;

/// The selected extent placement an extent rewrite redo names, if any.
///
/// An extent rewrite keeps the extent identity, so its redo carries the extent
/// id as both placements; an inline rewrite names a page generation there and
/// its record is never extent-placed.
pub(super) fn selected_source(
    selection: &PhysicalSourceSelection,
    rewrite: PhysicalRewriteRedo,
) -> Option<DurableExtentRecordPlacement> {
    let record = decode_record(rewrite.record_identity()).ok()?;
    selection
        .page_facts()
        .placements()
        .iter()
        .find_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Extent(extent)
                if extent.record() == record
                    && extent.extent().get() == rewrite.source_placement()
                    && rewrite.destination_placement() == rewrite.source_placement() =>
            {
                Some(*extent)
            }
            _ => None,
        })
}

/// Proves the selected root already publishes the rewrite's exact successor.
pub(super) fn prove(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    rewrite: PhysicalRewriteRedo,
    placement: DurableExtentRecordPlacement,
) -> Result<(), ()> {
    if !names_successor(rewrite)
        || placement.extent_generation() != rewrite.destination_generation()
    {
        return Err(());
    }
    let (manifest, payload) = read_generation(discovery, format, byte_limit, rewrite, placement)?;
    let (frames, expected) = encode_successor(format, rewrite, placement, &payload)?;
    let stored = read_bytes(discovery, byte_limit, placement)?;
    if manifest != expected || stored != frames.concat() {
        return Err(());
    }
    Ok(())
}

/// Rebuilds the successor generation from the verified source generation.
pub(super) fn project(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &PhysicalSourceSelection,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    admission: PhysicalRewriteAdmission,
    source: DurableExtentRecordPlacement,
) -> Result<PhysicalRedoProjection, ()> {
    let rewrite = admission.redo();
    if !names_successor(rewrite) || source.extent_generation() != rewrite.source_generation() {
        return Err(());
    }
    let (_, payload) = read_generation(discovery, format, byte_limit, rewrite, source)?;
    let (frames, manifest) = encode_successor(format, rewrite, source, &payload)?;
    let destination = DurableExtentRecordPlacement::new(
        source.record(),
        manifest.extent_cell(),
        source.payload_bytes(),
    )
    .ok_or(())?;
    let artifact = RecordArtifactFile::Extent {
        extent: source.extent().get(),
        generation: rewrite.destination_generation(),
    };
    let mut persisted = Vec::with_capacity(frames.len());
    let mut offset = 0_u64;
    let mut completed = 0_u64;
    let capacity = u64::from(manifest.chunk_payload_capacity());
    for (ordinal, bytes) in (1_u32..).zip(frames.iter()) {
        let coordinate = chunk_coordinate(manifest, completed, ordinal)?;
        let length = u32::try_from(bytes.len()).map_err(|_| ())?;
        persisted.push(
            PersistedPhysicalRecoveryFrame::new(
                PersistedPhysicalDataFrameSubject::ExtentChunk(coordinate),
                RecordFrameCoordinate::new(artifact, offset, length).ok_or(())?,
                bytes,
            )
            .ok_or(())?,
        );
        offset += u64::from(length);
        completed += (manifest.logical_bytes() - completed).min(capacity);
    }
    let manifest_file = PersistedPhysicalRecoveryManifest::new(
        RecordArtifactFile::ExtentManifest {
            extent: source.extent().get(),
            generation: rewrite.destination_generation(),
        },
        &manifest.encode(format),
    )
    .ok_or(())?;
    // No inline allocation or tail changes; the publication inherits both. The
    // allocation charge is the runtime's two-page streaming working set.
    let root_state = PersistedPhysicalRecoveryRootState::new(
        u64::from(format.page_size().bytes()) * 2,
        1,
        selection.root().selected().manifest().node_capacity(),
        Vec::new(),
        None,
        None,
    )
    .ok_or(())?;
    let projection = PersistedPhysicalRecoveryProjection::new(
        rewrite.source_root_generation(),
        root_state,
        vec![source.record()],
        persisted,
        vec![CurrentPhysicalRecordPlacement::Extent(destination)],
        Vec::new(),
        vec![manifest_file],
    )
    .ok_or(())?;
    Ok(PhysicalRedoProjection::from_rewrite_materialization(
        admission.operation(),
        admission.group(),
        admission.fate(),
        projection,
    ))
}

/// An extent rewrite always writes exactly the next generation.
fn names_successor(rewrite: PhysicalRewriteRedo) -> bool {
    rewrite.source_generation().checked_add(1) == Some(rewrite.destination_generation())
}

/// Reads one extent generation's manifest and payload, requiring the payload
/// to be exactly the bytes the redo digested.
fn read_generation(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    rewrite: PhysicalRewriteRedo,
    placement: DurableExtentRecordPlacement,
) -> Result<(DurableExtentManifest, Vec<u8>), ()> {
    let manifest_bytes = discovery
        .read_extent_manifest(
            placement.extent().get(),
            placement.extent_generation(),
            byte_limit,
        )
        .map_err(|_| ())?
        .into_bytes()
        .ok_or(())?;
    let (manifest, manifest_format) =
        DurableExtentManifest::decode(&manifest_bytes).map_err(|_| ())?;
    if manifest_format != format
        || manifest.record() != placement.record()
        || manifest.extent_cell() != placement.extent_cell()
        || manifest.logical_bytes() != placement.payload_bytes()
        || manifest.logical_bytes() != u64::from(rewrite.source_length())
    {
        return Err(());
    }
    let bytes = read_bytes(discovery, byte_limit, placement)?;
    let mut payload = Vec::with_capacity(rewrite.source_length() as usize);
    let mut offset = 0_usize;
    for ordinal in 1..=manifest.chunk_count() {
        let coordinate = chunk_coordinate(manifest, payload.len() as u64, ordinal)?;
        let length = (manifest.logical_bytes() as usize - payload.len())
            .min(manifest.chunk_payload_capacity() as usize);
        let frame_length = manifest.maximum_frame_bytes() as usize
            - manifest.chunk_payload_capacity() as usize
            + length;
        let frame = bytes.get(offset..offset + frame_length).ok_or(())?;
        let (chunk, chunk_format) = decode_extent_chunk(frame, coordinate).map_err(|_| ())?;
        if chunk_format != format {
            return Err(());
        }
        payload.extend_from_slice(chunk);
        offset += frame_length;
    }
    if offset != bytes.len() {
        return Err(());
    }
    let digest: [u8; 32] = Sha256::digest(&payload).into();
    if digest != rewrite.source_digest() {
        return Err(());
    }
    Ok((manifest, payload))
}

fn read_bytes(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    byte_limit: u64,
    placement: DurableExtentRecordPlacement,
) -> Result<Vec<u8>, ()> {
    discovery
        .read_extent(
            placement.extent().get(),
            placement.extent_generation(),
            byte_limit,
        )
        .map_err(|_| ())?
        .into_bytes()
        .ok_or(())
}

/// Encodes the successor generation's chunks, each stamped with the rewrite's
/// page LSN, exactly as the runtime bound them to the WAL.
fn encode_successor(
    format: PhysicalRecordFormatDeclaration,
    rewrite: PhysicalRewriteRedo,
    placement: DurableExtentRecordPlacement,
    payload: &[u8],
) -> Result<(Vec<Vec<u8>>, DurableExtentManifest), ()> {
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(placement.extent())
        .with_extent_generation(
            PhysicalGeneration::from_raw(rewrite.destination_generation()).map_err(|_| ())?,
        );
    let capacity = format.page_size().bytes() as usize
        - (worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES
            + worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES);
    let chunks = u32::try_from(payload.len().div_ceil(capacity)).map_err(|_| ())?;
    let manifest = DurableExtentManifest::new(
        format,
        placement.record(),
        cell,
        payload.len() as u64,
        format.page_size().bytes(),
        chunks,
    )
    .ok_or(())?;
    let mut frames = Vec::with_capacity(chunks as usize);
    let mut completed = 0_usize;
    for ordinal in 1..=chunks {
        let length = (payload.len() - completed).min(capacity);
        let coordinate = chunk_coordinate(manifest, completed as u64, ordinal)?;
        let mut chunk = prepare_extent_chunk(format, coordinate, length).map_err(|_| ())?;
        chunk
            .payload_mut()
            .copy_from_slice(&payload[completed..completed + length]);
        let mut bytes = chunk.seal();
        encode_data_frame_page_lsn(
            &mut bytes,
            DurableFrameKind::Extent,
            PhysicalPageLsn::new(rewrite.page_lsn()),
        )
        .map_err(|_| ())?;
        frames.push(bytes);
        completed += length;
    }
    Ok((frames, manifest))
}

fn chunk_coordinate(
    manifest: DurableExtentManifest,
    offset: u64,
    ordinal: u32,
) -> Result<ExtentChunkCoordinate, ()> {
    ExtentChunkCoordinate::new(
        manifest.record(),
        manifest.extent_cell(),
        manifest.logical_bytes(),
        offset,
        ordinal,
    )
    .ok_or(())
}
