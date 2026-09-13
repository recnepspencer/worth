use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    append_inline_records_owned, encode_data_frame_page_lsn, CurrentPhysicalRecordPlacement,
    DurableFrameKind, DurableInlineRecordPlacement, InlineRecordAppend,
    PersistedInlineSegmentAllocation, PersistedPhysicalDataFrameSubject,
    PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryProjection,
    PersistedPhysicalRecoveryRootState, PersistedRecordIdentity, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageId, PhysicalPageLsn, PhysicalRecordFormatDeclaration,
    PhysicalRecordSlot, PhysicalSegmentId, RecordArtifactFile, RecordFrameCoordinate,
    RecordSegmentPageManifestEntry,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use crate::{PhysicalRedoTargetIdentity, RecoveryPageObservation};

pub(super) fn range() -> WalLsnRange {
    WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(11)).unwrap()
}

fn target(generation: u64) -> PhysicalRedoTargetIdentity {
    PhysicalRedoTargetIdentity::InlinePage {
        segment: 1,
        page: 2,
        generation,
    }
}

pub(super) fn observation(
    generation: u64,
    page_lsn: u64,
    digest: [u8; 32],
) -> RecoveryPageObservation {
    RecoveryPageObservation::materialized(
        target(generation),
        page_lsn,
        digest,
        RecordFrameCoordinate::new(
            RecordArtifactFile::Segment {
                segment: 1,
                generation,
            },
            0,
            frame_len(),
        )
        .unwrap(),
        [3; 32],
    )
}

pub(super) fn encoded_redo() -> Vec<u8> {
    encoded_redo_with_segment_page_count(1)
}

pub(super) fn encoded_redo_with_segment_page_count(data_page_count: u32) -> Vec<u8> {
    let target = canonical_target_bytes();
    let projection = projection_with_segment_page_count(data_page_count);
    encoded_redo_with_projection(&target, projection)
}

pub(super) fn canonical_target_bytes() -> Vec<u8> {
    canonical_target_bytes_with_generations(2, 2)
}

pub(super) fn canonical_target_bytes_with_generations(
    page_generation: u64,
    artifact_generation: u64,
) -> Vec<u8> {
    let mut target = Vec::new();
    target.push(1);
    target.extend_from_slice(&1_u64.to_le_bytes());
    target.extend_from_slice(&2_u64.to_le_bytes());
    target.extend_from_slice(&page_generation.to_le_bytes());
    target.push(5);
    target.extend_from_slice(&1_u64.to_le_bytes());
    target.extend_from_slice(&artifact_generation.to_le_bytes());
    target.extend_from_slice(&0_u64.to_le_bytes());
    target.extend_from_slice(&frame_len().to_le_bytes());
    target
}

pub(super) fn projection_with_segment_page_count(
    data_page_count: u32,
) -> PersistedPhysicalRecoveryProjection {
    projection_with_generations(data_page_count, 2, 2)
}

pub(super) fn projection_with_generations(
    data_page_count: u32,
    page_generation: u64,
    artifact_generation: u64,
) -> PersistedPhysicalRecoveryProjection {
    projection_with_allocation(data_page_count, page_generation, artifact_generation, 1, 1)
}

pub(super) fn projection_with_allocation(
    data_page_count: u32,
    page_generation: u64,
    artifact_generation: u64,
    page_capacity: u32,
    used_pages: u32,
) -> PersistedPhysicalRecoveryProjection {
    projection_with_page_allocation(
        data_page_count,
        page_generation,
        artifact_generation,
        page_capacity,
        used_pages,
        2,
    )
}

pub(super) fn projection_with_page_allocation(
    data_page_count: u32,
    page_generation: u64,
    artifact_generation: u64,
    page_capacity: u32,
    used_pages: u32,
    page_id: u64,
) -> PersistedPhysicalRecoveryProjection {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let page = authority
        .page_cell(segment, PhysicalPageId::from_raw(page_id).unwrap())
        .with_page_generation(PhysicalGeneration::from_raw(page_generation).unwrap());
    let coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::Segment {
            segment: 1,
            generation: artifact_generation,
        },
        0,
        frame_len(),
    )
    .unwrap();
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::InlinePage(page),
        coordinate,
        &result_bytes_for_page(page_generation, page_id),
    )
    .unwrap();
    let record = PersistedRecordIdentity::new([1; 16], 1).unwrap();
    let slot = authority
        .slot_cell(
            segment,
            page.page_id(),
            PhysicalRecordSlot::from_raw(1).unwrap(),
        )
        .with_slot_generation(PhysicalGeneration::from_raw(1).unwrap());
    let segment_cell = authority
        .segment_cell(segment)
        .with_segment_generation(PhysicalGeneration::from_raw(artifact_generation).unwrap());
    let placement = DurableInlineRecordPlacement::new(
        record,
        segment_cell,
        page,
        slot,
        page_capacity,
        b"redo-record".len() as u64,
    )
    .unwrap();
    let routing =
        RecordSegmentPageManifestEntry::new(page, segment_cell, data_page_count, 0).unwrap();
    let root_state = PersistedPhysicalRecoveryRootState::new(
        4096,
        1,
        4,
        vec![
            PersistedInlineSegmentAllocation::new(segment_cell, page_capacity, used_pages).unwrap(),
        ],
        Some(record),
        Some(segment_cell),
    )
    .unwrap();
    PersistedPhysicalRecoveryProjection::new(
        1,
        root_state,
        vec![record],
        vec![frame],
        vec![CurrentPhysicalRecordPlacement::Inline(placement)],
        vec![routing],
        Vec::new(),
    )
    .unwrap()
}

pub(super) fn encoded_redo_with_projection(
    target: &[u8],
    projection: PersistedPhysicalRecoveryProjection,
) -> Vec<u8> {
    encoded_redo_with_projection_and_digest(target, projection, result_digest())
}

pub(super) fn encoded_redo_with_projection_and_digest(
    target: &[u8],
    projection: PersistedPhysicalRecoveryProjection,
    digest: [u8; 32],
) -> Vec<u8> {
    encoded_redo_with_projection_bytes_and_digest(target, &projection.encode(), digest)
}

pub(super) fn encoded_redo_with_projection_bytes(target: &[u8], projection: &[u8]) -> Vec<u8> {
    encoded_redo_with_projection_bytes_and_digest(target, projection, result_digest())
}

pub(super) fn encoded_redo_with_projection_bytes_and_digest(
    target: &[u8],
    projection: &[u8],
    digest: [u8; 32],
) -> Vec<u8> {
    let mut encoded = Vec::new();
    field(&mut encoded, b"store.physical.wal.canonical-redo.v3");
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    encoded.extend_from_slice(&0_u32.to_le_bytes());
    encoded.extend_from_slice(&10_u64.to_le_bytes());
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    field(&mut encoded, target);
    encoded.extend_from_slice(&digest);
    field(&mut encoded, b"redo-record");
    field(&mut encoded, projection);
    encoded
}

pub(super) fn result_bytes() -> Vec<u8> {
    result_bytes_with_page_generation(2)
}

fn result_bytes_with_page_generation(page_generation: u64) -> Vec<u8> {
    result_bytes_for_page(page_generation, 2)
}

pub(super) fn result_bytes_for_page(page_generation: u64, page_id: u64) -> Vec<u8> {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let page = authority
        .page_cell(segment, PhysicalPageId::from_raw(page_id).unwrap())
        .with_page_generation(PhysicalGeneration::from_raw(page_generation).unwrap());
    let record = PersistedRecordIdentity::new([1; 16], 1).unwrap();
    let slot = authority
        .slot_cell(
            segment,
            page.page_id(),
            PhysicalRecordSlot::from_raw(1).unwrap(),
        )
        .with_slot_generation(PhysicalGeneration::from_raw(1).unwrap());
    let mut bytes = append_inline_records_owned(
        format,
        page,
        None,
        &[InlineRecordAppend::new(record, slot, b"redo-record")],
    )
    .unwrap()
    .0;
    encode_data_frame_page_lsn(
        &mut bytes,
        DurableFrameKind::InlinePage,
        PhysicalPageLsn::new(10),
    )
    .unwrap();
    bytes
}

pub(super) fn result_digest() -> [u8; 32] {
    result_digest_for_page_generation(2)
}

pub(super) fn result_digest_for_page_generation(page_generation: u64) -> [u8; 32] {
    Sha256::digest(result_bytes_with_page_generation(page_generation)).into()
}

pub(super) fn frame_len() -> u32 {
    PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap()
        .page_size()
        .bytes()
}

fn field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}

pub(super) fn replace_first(bytes: &mut [u8], old: &[u8], new: &[u8]) {
    assert_eq!(old.len(), new.len());
    let offset = bytes
        .windows(old.len())
        .position(|window| window == old)
        .expect("fixture contains the governed field");
    bytes[offset..offset + old.len()].copy_from_slice(new);
}
