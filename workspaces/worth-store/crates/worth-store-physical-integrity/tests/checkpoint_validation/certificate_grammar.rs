use worth_store_physical_format::{
    encode_checkpoint_certificate, CheckpointCertificateKind, CheckpointSelectiveRecordAggregate,
    OriginalDropReservationRequestV1, PersistedRecordIdentity, ReleaseCheckpointAccumulatorV1,
    ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1, ReleaseCheckpointNoReleaseV1,
    ReleasedDropWalFateWitnessV1,
};
use worth_store_physical_integrity::{
    validate_checkpoint_footer, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange,
    PhysicalDamageCause, UntrustedPhysicalArtifact,
};

use super::literal_vectors::{BINDING, BINDING_COMPACTION, DIRTY_BASIS, FOOTER, HEADER};
use super::support::{
    assert_damage, binding_scope, compaction_scope, dirty_scope, header_scope_staged, identity,
    reseal_record, validate_binding, validate_compaction, validate_dirty, validate_header,
    FOOTER_OFFSET,
};

#[test]
fn encoded_v2_accumulator_only_footer_is_admitted() {
    assert_certificate_sequence_admitted(&[v2_accumulator_only_frame()]);
}

#[test]
fn encoded_batch_followed_by_v2_accumulator_footer_is_admitted() {
    assert_certificate_sequence_admitted(&[batch_frame(), v2_accumulator_after_batch_frame()]);
}

#[test]
fn duplicate_and_mixed_v2_accumulators_are_denied() {
    let v2 = v2_accumulator_only_frame();
    let v1 = v1_accumulator_frame();
    assert_certificate_sequence_denied(&[v2.clone(), v2.clone()]);
    assert_certificate_sequence_denied(&[v1.clone(), v2.clone()]);
    assert_certificate_sequence_denied(&[v2.clone(), v1]);
    let no_release = no_release_frame();
    assert_certificate_sequence_denied(&[no_release.clone(), v2.clone()]);
    assert_certificate_sequence_denied(&[v2, no_release]);
}

#[test]
fn duplicate_well_framed_no_release_is_denied() {
    let frame = no_release_frame();
    assert_certificate_sequence_denied(&[frame.clone(), frame]);
}

#[test]
fn well_framed_no_release_cannot_mix_with_batch_or_accumulator() {
    let no_release = no_release_frame();
    let batch = ReleaseCheckpointBatchV1::new(
        identity(),
        15,
        [1; 32],
        0,
        record(4),
        [2; 32],
        [3; 32],
        record(5),
        [4; 32],
        OriginalDropReservationRequestV1::new([5; 32], [6; 32], 10, 20).unwrap(),
        ReleasedDropWalFateWitnessV1::new(100, 110, [7; 32], [8; 32]).unwrap(),
        12,
        [9; 32],
        None,
        2,
        [10; 32],
        false,
    )
    .unwrap();
    let accumulator = ReleaseCheckpointAccumulatorV1::new(
        identity(),
        15,
        [1; 32],
        0,
        [0; 32],
        [0; 32],
        0,
        [0; 32],
        1,
        [11; 32],
        batch.tip_provenance().unwrap(),
        2,
        [10; 32],
        false,
    )
    .unwrap();
    for other in [
        release_frame(&batch.encode()),
        release_frame(&accumulator.encode()),
    ] {
        assert_certificate_sequence_denied(&[no_release.clone(), other.clone()]);
        assert_certificate_sequence_denied(&[other, no_release.clone()]);
    }
}

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([9; 16], ordinal).unwrap()
}

fn batch() -> ReleaseCheckpointBatchV1 {
    ReleaseCheckpointBatchV1::new(
        identity(),
        15,
        [1; 32],
        0,
        record(4),
        [2; 32],
        [3; 32],
        record(5),
        [4; 32],
        OriginalDropReservationRequestV1::new([5; 32], [6; 32], 10, 20).unwrap(),
        ReleasedDropWalFateWitnessV1::new(100, 110, [7; 32], [8; 32]).unwrap(),
        12,
        [9; 32],
        None,
        2,
        [10; 32],
        false,
    )
    .unwrap()
}

fn batch_frame() -> Vec<u8> {
    release_frame(&batch().encode())
}

fn v1_accumulator_frame() -> Vec<u8> {
    let accumulator = ReleaseCheckpointAccumulatorV1::new(
        identity(),
        15,
        [1; 32],
        0,
        [0; 32],
        [0; 32],
        0,
        [0; 32],
        1,
        [11; 32],
        batch().tip_provenance().unwrap(),
        2,
        [10; 32],
        false,
    )
    .unwrap();
    release_frame(&accumulator.encode())
}

fn v2_accumulator_only_frame() -> Vec<u8> {
    let carried = ReleaseCheckpointAccumulatorV1::new(
        identity(),
        15,
        [1; 32],
        6,
        [12; 32],
        [13; 32],
        2,
        [10; 32],
        0,
        [0; 32],
        batch().tip_provenance().unwrap(),
        2,
        [10; 32],
        false,
    )
    .unwrap();
    let v2 = ReleaseCheckpointAccumulatorV2::new(carried, 1, [17; 32], 1, [16; 32]).unwrap();
    release_frame(&v2.encode())
}

fn v2_accumulator_after_batch_frame() -> Vec<u8> {
    let v1 = ReleaseCheckpointAccumulatorV1::new(
        identity(),
        15,
        [1; 32],
        0,
        [0; 32],
        [0; 32],
        0,
        [0; 32],
        1,
        [11; 32],
        batch().tip_provenance().unwrap(),
        2,
        [10; 32],
        false,
    )
    .unwrap();
    let v2 = ReleaseCheckpointAccumulatorV2::new(v1, 1, [17; 32], 0, [0; 32]).unwrap();
    release_frame(&v2.encode())
}

fn no_release_frame() -> Vec<u8> {
    let header = validate_header(&HEADER, header_scope_staged());
    let marker = ReleaseCheckpointNoReleaseV1::new(
        identity(),
        header.source().root().generation(),
        [1; 32],
        0,
        [0; 32],
        [0; 32],
    )
    .unwrap();
    release_frame(&marker.encode())
}

fn release_frame(payload: &[u8]) -> Vec<u8> {
    let frame =
        encode_checkpoint_certificate(CheckpointCertificateKind::ReleasedDrop, payload).unwrap();
    assert_eq!(
        worth_store_physical_format::decode_checkpoint_certificate(&frame).unwrap(),
        (CheckpointCertificateKind::ReleasedDrop, payload)
    );
    frame
}

fn assert_certificate_sequence_admitted(frames: &[Vec<u8>]) {
    let checkpoint = identity();
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(checkpoint));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(checkpoint));
    let binding = validate_binding(&BINDING, binding_scope(checkpoint, BINDING.len() as u64));
    let mut next_offset = FOOTER_OFFSET;
    let certificates: Vec<_> = frames
        .iter()
        .map(|frame| {
            let range = PhysicalByteRange::new(next_offset, frame.len() as u64).unwrap();
            next_offset = range.end_exclusive();
            (range, frame.as_slice())
        })
        .collect();
    let mut aggregate = CheckpointSelectiveRecordAggregate::new();
    for frame in frames {
        aggregate.include(frame).unwrap();
    }
    let summary = aggregate.summary();
    let mut footer = FOOTER[..FOOTER.len() - 4].to_vec();
    footer[8] = worth_store_physical_format::CHECKPOINT_CERTIFIED_SCHEMA;
    footer[12..16].copy_from_slice(&184_u32.to_le_bytes());
    footer.extend_from_slice(&summary.record_count().to_le_bytes());
    footer.extend_from_slice(&summary.encoded_bytes().to_le_bytes());
    footer.extend_from_slice(&summary.digest());
    footer.extend_from_slice(&[0; 4]);
    reseal_record(&mut footer);
    let scope = PhysicalArtifactScope::checkpoint_footer(
        checkpoint,
        PhysicalByteRange::new(next_offset, footer.len() as u64).unwrap(),
    );
    let basis = CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    )
    .with_certificates(&certificates);
    assert!(matches!(
        validate_checkpoint_footer(
            UntrustedPhysicalArtifact::from_bounded_bytes(&footer),
            scope,
            basis
        )
        .0,
        CheckpointFooterIntegrityValidation::Intact(_)
    ));
}

fn assert_certificate_sequence_denied(frames: &[Vec<u8>]) {
    let checkpoint = identity();
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(checkpoint));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(checkpoint));
    let binding = validate_binding(&BINDING, binding_scope(checkpoint, BINDING.len() as u64));
    let mut next_offset = FOOTER_OFFSET;
    let certificates: Vec<_> = frames
        .iter()
        .map(|frame| {
            let range = PhysicalByteRange::new(next_offset, frame.len() as u64).unwrap();
            next_offset = range.end_exclusive();
            (range, frame.as_slice())
        })
        .collect();
    let scope = PhysicalArtifactScope::checkpoint_footer(
        checkpoint,
        PhysicalByteRange::new(next_offset, FOOTER.len() as u64).unwrap(),
    );
    let basis = CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    )
    .with_certificates(&certificates);
    let (validation, _) = validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    );
    let CheckpointFooterIntegrityValidation::Rejected(rejection) = validation else {
        panic!("ambiguous release certificate roster unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::SequenceMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    );
}
