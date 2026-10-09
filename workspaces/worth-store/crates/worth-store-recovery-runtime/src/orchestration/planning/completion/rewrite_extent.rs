use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{BoundedRecoveryFilesystemDiscovery, ReadGrant, UnchargedRead};
use worth_store_physical_format::{
    decode_extent_chunk, encode_data_frame_page_lsn, prepare_extent_chunk,
    CurrentPhysicalRecordPlacement, DurableExtentManifest, DurableExtentRecordPlacement,
    DurableFrameKind, ExtentArenaFrameLayout, ExtentChunkFrame, PersistedPhysicalDataFrameSubject,
    PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryManifest,
    PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageLsn, PhysicalRecordFormatDeclaration,
    PhysicalRewriteRedo, RecordArtifactFile, RecordFrameCoordinate,
};
use worth_store_recovery_physics::{
    PhysicalRedoProjection, PhysicalRewriteAdmission, PhysicalSourceSelection,
};

use super::super::historical_publication::{discovery_failure, HistoricalFailure};
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
    let record = decode_record(rewrite.record_identity())?;
    rewrite.extent_arena()?;
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
    rewrite: PhysicalRewriteRedo,
    placement: DurableExtentRecordPlacement,
) -> Result<(), HistoricalFailure> {
    if !names_successor(rewrite)
        || placement.extent_generation() != rewrite.destination_generation()
        || placement.arena_range()
            != rewrite
                .extent_arena()
                .ok_or(HistoricalFailure::Invalid)?
                .destination()
    {
        return Err(HistoricalFailure::Invalid);
    }
    let (manifest, payload) = read_generation(discovery, format, rewrite, placement)?;
    let (frames, expected) = encode_successor(format, rewrite, placement, &payload)?;
    if manifest != expected {
        return Err(HistoricalFailure::Invalid);
    }
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment())
        .ok_or(HistoricalFailure::Invalid)?;
    for (ordinal, frame) in (1_u32..).zip(frames) {
        let chunk =
            ExtentChunkFrame::of(manifest, layout, ordinal).ok_or(HistoricalFailure::Invalid)?;
        if frame.len() != chunk.length() as usize {
            return Err(HistoricalFailure::Invalid);
        }
        let observed = discovery
            .read_extent_range(
                placement.arena_range(),
                chunk.offset(),
                chunk.length(),
                ReadGrant::ceiling_only(),
            )
            .observed()
            .map_err(discovery_failure)?
            .into_bytes()
            .ok_or(HistoricalFailure::Invalid)?;
        if observed != frame {
            return Err(HistoricalFailure::Invalid);
        }
    }
    Ok(())
}

/// Rebuilds the successor generation from the verified source generation.
pub(super) fn project(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &PhysicalSourceSelection,
    format: PhysicalRecordFormatDeclaration,
    admission: PhysicalRewriteAdmission,
    source: DurableExtentRecordPlacement,
) -> Result<PhysicalRedoProjection, HistoricalFailure> {
    let rewrite = admission.redo();
    let arena = rewrite.extent_arena().ok_or(HistoricalFailure::Invalid)?;
    if !names_successor(rewrite) || source.extent_generation() != rewrite.source_generation() {
        return Err(HistoricalFailure::Invalid);
    }
    if source.arena_range() != arena.source() {
        return Err(HistoricalFailure::Invalid);
    }
    let (_, payload) = read_generation(discovery, format, rewrite, source)?;
    let (frames, manifest) = encode_successor(format, rewrite, source, &payload)?;
    let destination = DurableExtentRecordPlacement::new_selected(
        source.record(),
        manifest.extent_cell(),
        source.payload_bytes(),
        arena.destination(),
        source.route_metadata(),
    )
    .ok_or(HistoricalFailure::Invalid)?;
    let artifact = RecordArtifactFile::ExtentArena {
        arena: arena.destination().arena().get(),
    };
    let mut persisted = Vec::with_capacity(frames.len());
    let layout =
        ExtentArenaFrameLayout::new(format, arena.alignment()).ok_or(HistoricalFailure::Invalid)?;
    for (ordinal, bytes) in (1_u32..).zip(frames.iter()) {
        let chunk =
            ExtentChunkFrame::of(manifest, layout, ordinal).ok_or(HistoricalFailure::Invalid)?;
        if bytes.len() != chunk.length() as usize {
            return Err(HistoricalFailure::Invalid);
        }
        persisted.push(
            PersistedPhysicalRecoveryFrame::new(
                PersistedPhysicalDataFrameSubject::ExtentChunk(chunk.coordinate()),
                RecordFrameCoordinate::new(
                    artifact,
                    arena.destination().offset() + chunk.offset(),
                    chunk.length(),
                )
                .ok_or(HistoricalFailure::Invalid)?,
                bytes,
            )
            .ok_or(HistoricalFailure::Invalid)?,
        );
    }
    let manifest_file = PersistedPhysicalRecoveryManifest::new(
        RecordFrameCoordinate::new(artifact, arena.destination().offset(), 104)
            .ok_or(HistoricalFailure::Invalid)?,
        &manifest.encode(format),
    )
    .ok_or(HistoricalFailure::Invalid)?;
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
    .ok_or(HistoricalFailure::Invalid)?;
    let projection = PersistedPhysicalRecoveryProjection::new(
        rewrite.source_root_generation(),
        root_state,
        vec![source.record()],
        persisted,
        vec![CurrentPhysicalRecordPlacement::Extent(destination)],
        Vec::new(),
        vec![manifest_file],
    )
    .ok_or(HistoricalFailure::Invalid)?;
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
    rewrite: PhysicalRewriteRedo,
    placement: DurableExtentRecordPlacement,
) -> Result<(DurableExtentManifest, Vec<u8>), HistoricalFailure> {
    let manifest_bytes = discovery
        .read_extent_manifest(placement.arena_range(), ReadGrant::ceiling_only())
        .observed()
        .map_err(discovery_failure)?
        .into_bytes()
        .ok_or(HistoricalFailure::Invalid)?;
    let (manifest, manifest_format) =
        DurableExtentManifest::decode(&manifest_bytes).map_err(|_| HistoricalFailure::Invalid)?;
    if manifest_format != format
        || manifest.record() != placement.record()
        || manifest.extent_cell() != placement.extent_cell()
        || manifest.logical_bytes() != placement.payload_bytes()
        || manifest.logical_bytes() != u64::from(rewrite.source_length())
        || manifest.alignment()
            != rewrite
                .extent_arena()
                .ok_or(HistoricalFailure::Invalid)?
                .alignment()
    {
        return Err(HistoricalFailure::Invalid);
    }
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment())
        .ok_or(HistoricalFailure::Invalid)?;
    if layout
        .allocated_bytes(manifest.chunk_count())
        .ok_or(HistoricalFailure::Invalid)?
        != placement.arena_range().length()
    {
        return Err(HistoricalFailure::Invalid);
    }
    let mut payload = Vec::with_capacity(rewrite.source_length() as usize);
    for ordinal in 1..=manifest.chunk_count() {
        let chunk =
            ExtentChunkFrame::of(manifest, layout, ordinal).ok_or(HistoricalFailure::Invalid)?;
        let frame = discovery
            .read_extent_range(
                placement.arena_range(),
                chunk.offset(),
                chunk.length(),
                ReadGrant::ceiling_only(),
            )
            .observed()
            .map_err(discovery_failure)?
            .into_bytes()
            .ok_or(HistoricalFailure::Invalid)?;
        let (chunk, chunk_format) = decode_extent_chunk(&frame, chunk.coordinate())
            .map_err(|_| HistoricalFailure::Invalid)?;
        if chunk_format != format {
            return Err(HistoricalFailure::Invalid);
        }
        payload.extend_from_slice(chunk);
    }
    let digest: [u8; 32] = Sha256::digest(&payload).into();
    if digest != rewrite.source_digest() {
        return Err(HistoricalFailure::Invalid);
    }
    Ok((manifest, payload))
}

/// Encodes the successor generation's chunks, each stamped with the rewrite's
/// page LSN, exactly as the runtime bound them to the WAL.
fn encode_successor(
    format: PhysicalRecordFormatDeclaration,
    rewrite: PhysicalRewriteRedo,
    placement: DurableExtentRecordPlacement,
    payload: &[u8],
) -> Result<(Vec<Vec<u8>>, DurableExtentManifest), HistoricalFailure> {
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(placement.extent())
        .with_extent_generation(
            PhysicalGeneration::from_raw(rewrite.destination_generation())
                .map_err(|_| HistoricalFailure::Invalid)?,
        );
    let capacity = format.page_size().bytes() as usize
        - (worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES
            + worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES);
    let chunks =
        u32::try_from(payload.len().div_ceil(capacity)).map_err(|_| HistoricalFailure::Invalid)?;
    let alignment = rewrite
        .extent_arena()
        .ok_or(HistoricalFailure::Invalid)?
        .alignment();
    let manifest = DurableExtentManifest::new(
        format,
        placement.record(),
        cell,
        payload.len() as u64,
        format.page_size().bytes(),
        chunks,
        alignment,
    )
    .ok_or(HistoricalFailure::Invalid)?;
    let layout =
        ExtentArenaFrameLayout::new(format, alignment).ok_or(HistoricalFailure::Invalid)?;
    let mut frames = Vec::with_capacity(chunks as usize);
    let mut completed = 0_usize;
    for ordinal in 1..=chunks {
        let framed =
            ExtentChunkFrame::of(manifest, layout, ordinal).ok_or(HistoricalFailure::Invalid)?;
        let length = framed.payload_bytes() as usize;
        let mut chunk = prepare_extent_chunk(format, framed.coordinate(), length)
            .map_err(|_| HistoricalFailure::Invalid)?;
        chunk
            .payload_mut()
            .copy_from_slice(&payload[completed..completed + length]);
        let mut bytes = chunk.seal();
        encode_data_frame_page_lsn(
            &mut bytes,
            DurableFrameKind::Extent,
            PhysicalPageLsn::new(rewrite.page_lsn()),
        )
        .map_err(|_| HistoricalFailure::Invalid)?;
        frames.push(bytes);
        completed += length;
    }
    Ok((frames, manifest))
}
