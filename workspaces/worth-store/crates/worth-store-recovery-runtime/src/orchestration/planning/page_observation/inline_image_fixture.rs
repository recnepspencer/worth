//! Canonically encoded inline page images for page-observation tests.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::recovery_wal::{LogSequenceNumber, WalLsnRange};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableInlineRecordPlacement, PersistedInlineSegmentAllocation,
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState,
    PersistedRecordIdentity, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalRecordFormatDeclaration, PhysicalRecordSlot, PhysicalSegmentId, RecordArtifactFile,
    RecordFrameCoordinate, RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::{decode_physical_redo_records, PhysicalRedoTarget};

/// The projection of a member that wrote one image of an inline page holding
/// `records`, which may be none, and the WAL target of that image as real redo
/// decoding names it.
pub(super) fn inline_image(
    page: (u64, u64, u64),
    artifact: (u64, u64),
    records: &[PersistedRecordIdentity],
    fill: u8,
) -> (PersistedPhysicalRecoveryProjection, PhysicalRedoTarget) {
    image(page, artifact, records, None, fill)
}

/// As [`inline_image`], from a member that also placed the `stray` record on
/// the named other generation of the same page: a placement the image it
/// framed does not hold.
pub(super) fn inline_image_beside_stray(
    page: (u64, u64, u64),
    artifact: (u64, u64),
    records: &[PersistedRecordIdentity],
    stray: (PersistedRecordIdentity, u64),
    fill: u8,
) -> (PersistedPhysicalRecoveryProjection, PhysicalRedoTarget) {
    image(page, artifact, records, Some(stray), fill)
}

fn image(
    page: (u64, u64, u64),
    artifact: (u64, u64),
    records: &[PersistedRecordIdentity],
    stray: Option<(PersistedRecordIdentity, u64)>,
    fill: u8,
) -> (PersistedPhysicalRecoveryProjection, PhysicalRedoTarget) {
    let bytes = [fill; 8];
    let projection = projection(page, artifact, records, stray, &bytes);
    let target = decoded_target(page, artifact, &bytes, &projection);
    (projection, target)
}

/// What the member logged: the image frame, one placement for every record
/// the image holds, the stray placement if any, and the routing of the page.
fn projection(
    (segment, page, page_generation): (u64, u64, u64),
    (artifact_segment, artifact_generation): (u64, u64),
    records: &[PersistedRecordIdentity],
    stray: Option<(PersistedRecordIdentity, u64)>,
    bytes: &[u8],
) -> PersistedPhysicalRecoveryProjection {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment_id = PhysicalSegmentId::from_raw(segment).unwrap();
    let page_cell = authority
        .page_cell(segment_id, PhysicalPageId::from_raw(page).unwrap())
        .with_page_generation(PhysicalGeneration::from_raw(page_generation).unwrap());
    let coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::Segment {
            segment: artifact_segment,
            generation: artifact_generation,
        },
        0,
        bytes.len() as u32,
    )
    .unwrap();
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::InlinePage(page_cell),
        coordinate,
        bytes,
    )
    .unwrap();
    let artifact_cell = authority
        .segment_cell(PhysicalSegmentId::from_raw(artifact_segment).unwrap())
        .with_segment_generation(PhysicalGeneration::from_raw(artifact_generation).unwrap());
    let place = |record: PersistedRecordIdentity, slot: u16, generation: u64| {
        let placed_on = authority
            .page_cell(segment_id, page_cell.page_id())
            .with_page_generation(PhysicalGeneration::from_raw(generation).unwrap());
        let slot = authority
            .slot_cell(
                segment_id,
                page_cell.page_id(),
                PhysicalRecordSlot::from_raw(slot).unwrap(),
            )
            .with_slot_generation(PhysicalGeneration::from_raw(1).unwrap());
        CurrentPhysicalRecordPlacement::Inline(
            DurableInlineRecordPlacement::legacy_unknown(
                record,
                artifact_cell,
                placed_on,
                slot,
                4,
                4,
            )
            .unwrap(),
        )
    };
    let mut placements = records
        .iter()
        .zip(1_u16..)
        .map(|(record, slot)| place(*record, slot, page_generation))
        .collect::<Vec<_>>();
    if let Some((record, generation)) = stray {
        placements.push(place(record, records.len() as u16 + 1, generation));
    }
    // A member always names a record. One that places nothing on its image
    // names a record the image does not hold.
    let appended = records
        .last()
        .copied()
        .unwrap_or_else(|| PersistedRecordIdentity::new([0xfe; 16], 1).unwrap());
    let routing = RecordSegmentPageManifestEntry::new(page_cell, artifact_cell, 1, 0).unwrap();
    PersistedPhysicalRecoveryProjection::new(
        1,
        PersistedPhysicalRecoveryRootState::new(
            4096,
            1,
            4,
            vec![PersistedInlineSegmentAllocation::new(artifact_cell, 4, 1).unwrap()],
            Some(appended),
            Some(artifact_cell),
        )
        .unwrap(),
        vec![appended],
        vec![frame],
        placements,
        vec![routing],
        Vec::new(),
    )
    .unwrap()
}

/// The target of the image as real redo decoding names it from the canonical
/// encoding of the member that logged this projection.
fn decoded_target(
    (segment, page, page_generation): (u64, u64, u64),
    (artifact_segment, artifact_generation): (u64, u64),
    bytes: &[u8],
    projection: &PersistedPhysicalRecoveryProjection,
) -> PhysicalRedoTarget {
    let mut target = Vec::new();
    target.push(1);
    target.extend_from_slice(&segment.to_le_bytes());
    target.extend_from_slice(&page.to_le_bytes());
    target.extend_from_slice(&page_generation.to_le_bytes());
    target.push(5);
    target.extend_from_slice(&artifact_segment.to_le_bytes());
    target.extend_from_slice(&artifact_generation.to_le_bytes());
    target.extend_from_slice(&0_u64.to_le_bytes());
    target.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    let mut encoded = Vec::new();
    field(&mut encoded, b"store.physical.wal.canonical-redo.v3");
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    encoded.extend_from_slice(&0_u32.to_le_bytes());
    encoded.extend_from_slice(&10_u64.to_le_bytes());
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    field(&mut encoded, &target);
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    encoded.extend_from_slice(&digest);
    field(&mut encoded, b"redo");
    field(&mut encoded, &projection.encode());
    decode_physical_redo_records(
        &encoded,
        WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(11)).unwrap(),
        projection.placements().len().max(1) as u64,
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    )
    .unwrap()[0]
        .targets()[0]
        .clone()
}

fn field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}
