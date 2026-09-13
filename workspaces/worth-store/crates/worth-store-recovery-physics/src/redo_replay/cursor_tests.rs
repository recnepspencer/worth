use super::*;
use crate::{decode_physical_redo_records, PhysicalRedoPlanningDenial};
use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    PersistedInlineSegmentAllocation, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState, RecordArtifactFile,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

#[test]
fn cursor_advances_exact_successors_and_rejects_same_generation_cross_operation() {
    let first = decoded_target(2);
    let second = decoded_target(3);
    let prior = RecoveryPageObservation::materialized(
        PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page: 2,
            generation: 1,
        },
        9,
        [1; 32],
        RecordFrameCoordinate::new(
            RecordArtifactFile::Segment {
                segment: 1,
                generation: 1,
            },
            4096,
            4096,
        )
        .unwrap(),
        [4; 32],
    );
    let mut cursor = RecoveryPageCursor::new(vec![prior]).unwrap();
    assert_eq!(
        cursor.observe(second.identity()),
        Err(PhysicalRedoPlanningDenial::GenerationMismatch),
        "MUTANT_PREDICATE:c8-redo-target-generation-ignored"
    );
    cursor.advance([5; 32], &first, 10).unwrap();
    cursor.advance([6; 32], &second, 11).unwrap();

    let mut same_generation = RecoveryPageCursor::new(vec![prior]).unwrap();
    same_generation.advance([5; 32], &first, 10).unwrap();
    assert_eq!(
        same_generation.advance([6; 32], &first, 11),
        Err(PhysicalRedoPlanningDenial::GenerationMismatch),
        "MUTANT_PREDICATE:c8-redo-target-generation-ignored"
    );
    same_generation
        .advance([5; 32], &first, 11)
        .expect("MUTANT_PREDICATE:c8-nonidempotent-redo-repeated");
}

#[test]
fn absent_prior_digest_is_the_shared_c7_producer_grammar() {
    let target = decoded_target(1);
    let observation = RecoveryPageObservation::absent(&target, [8; 32]);
    let (subject, coordinate) = target_format_basis(&target);
    assert_eq!(
        observation.frame_digest(),
        worth_store_physical_format::certified_absent_prior_image_digest(subject, coordinate)
    );
    assert_eq!(observation.page_lsn(), 0);
    assert!(observation.is_absent_prior());
}

fn decoded_target(generation: u64) -> PhysicalRedoTarget {
    let frame_bytes = vec![generation as u8; 4096];
    let digest: [u8; 32] = Sha256::digest(&frame_bytes).into();
    let mut target = Vec::new();
    target.push(1);
    target.extend_from_slice(&1_u64.to_le_bytes());
    target.extend_from_slice(&2_u64.to_le_bytes());
    target.extend_from_slice(&generation.to_le_bytes());
    target.push(5);
    target.extend_from_slice(&1_u64.to_le_bytes());
    target.extend_from_slice(&generation.to_le_bytes());
    target.extend_from_slice(&0_u64.to_le_bytes());
    target.extend_from_slice(&4096_u32.to_le_bytes());
    let mut encoded = Vec::new();
    field(&mut encoded, b"store.physical.wal.canonical-redo.v3");
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    encoded.extend_from_slice(&0_u32.to_le_bytes());
    encoded.extend_from_slice(&10_u64.to_le_bytes());
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    field(&mut encoded, &target);
    encoded.extend_from_slice(&digest);
    field(&mut encoded, b"redo");
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let page = authority
        .page_cell(segment, PhysicalPageId::from_raw(2).unwrap())
        .with_page_generation(PhysicalGeneration::from_raw(generation).unwrap());
    let coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::Segment {
            segment: 1,
            generation,
        },
        0,
        4096,
    )
    .unwrap();
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::InlinePage(page),
        coordinate,
        &frame_bytes,
    )
    .unwrap();
    let record = PersistedRecordIdentity::new([1; 16], 1).unwrap();
    let segment_cell = authority
        .segment_cell(segment)
        .with_segment_generation(PhysicalGeneration::from_raw(generation).unwrap());
    let root_state = PersistedPhysicalRecoveryRootState::new(
        1,
        1,
        2,
        vec![PersistedInlineSegmentAllocation::new(segment_cell, 1, 1).unwrap()],
        Some(record),
        Some(segment_cell),
    )
    .unwrap();
    let projection = PersistedPhysicalRecoveryProjection::new(
        generation.saturating_sub(1).max(1),
        root_state,
        vec![record],
        vec![frame],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    field(&mut encoded, &projection.encode());
    let range = WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(11)).unwrap();
    decode_physical_redo_records(&encoded, range, 1).unwrap()[0].targets()[0].clone()
}

fn field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}
