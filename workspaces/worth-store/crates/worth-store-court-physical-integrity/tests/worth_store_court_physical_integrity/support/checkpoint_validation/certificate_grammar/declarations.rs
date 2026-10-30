pub(crate) use worth_store_physical_format::{
    encode_checkpoint_certificate, CheckpointCertificateKind, CheckpointSelectiveRecordAggregate,
    OriginalDropReservationRequestV1, PersistedRecordIdentity, ReleaseCheckpointAccumulatorV1,
    ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1, ReleaseCheckpointNoReleaseV1,
    ReleasedDropWalFateWitnessV1,
};
pub(crate) use worth_store_physical_integrity::{
    validate_checkpoint_footer, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange,
    PhysicalDamageCause, UntrustedPhysicalArtifact,
};

pub(crate) use crate::checkpoint_validation::support::{
    assert_damage, binding_scope, compaction_scope, dirty_scope, header_scope_staged, identity,
    reseal_record, validate_binding, validate_compaction, validate_dirty, validate_header,
    FOOTER_OFFSET,
};
pub(crate) use crate::literal_vectors::checkpoint_validation::{
    BINDING, BINDING_COMPACTION, DIRTY_BASIS, FOOTER, HEADER,
};

pub(crate) fn record(ordinal: u64) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new([9; 16], ordinal).unwrap()
}

pub(crate) fn batch() -> ReleaseCheckpointBatchV1 {
    worth_store_physical_format::ReleaseCheckpointBatchV1::new(
        identity(),
        15,
        [1; 32],
        0,
        record(4),
        [2; 32],
        [3; 32],
        record(5),
        [4; 32],
        worth_store_physical_format::OriginalDropReservationRequestV1::new(
            [5; 32], [6; 32], 10, 20,
        )
        .unwrap(),
        worth_store_physical_format::ReleasedDropWalFateWitnessV1::new(100, 110, [7; 32], [8; 32])
            .unwrap(),
        12,
        [9; 32],
        None,
        2,
        [10; 32],
        false,
    )
    .unwrap()
}

pub(crate) fn batch_frame() -> Vec<u8> {
    release_frame(&batch().encode())
}

pub(crate) fn v1_accumulator_frame() -> Vec<u8> {
    let accumulator = worth_store_physical_format::ReleaseCheckpointAccumulatorV1::new(
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

pub(crate) fn v2_accumulator_only_frame() -> Vec<u8> {
    let carried = worth_store_physical_format::ReleaseCheckpointAccumulatorV1::new(
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
    let v2 = worth_store_physical_format::ReleaseCheckpointAccumulatorV2::new(
        carried, 1, [17; 32], 1, [16; 32],
    )
    .unwrap();
    release_frame(&v2.encode())
}

pub(crate) fn v2_accumulator_after_batch_frame() -> Vec<u8> {
    let v1 = worth_store_physical_format::ReleaseCheckpointAccumulatorV1::new(
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
    let v2 = worth_store_physical_format::ReleaseCheckpointAccumulatorV2::new(
        v1, 1, [17; 32], 0, [0; 32],
    )
    .unwrap();
    release_frame(&v2.encode())
}

pub(crate) fn no_release_frame() -> Vec<u8> {
    let header = validate_header(&HEADER, header_scope_staged());
    let marker = worth_store_physical_format::ReleaseCheckpointNoReleaseV1::new(
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

pub(crate) fn release_frame(payload: &[u8]) -> Vec<u8> {
    let frame = encode_checkpoint_certificate(
        worth_store_physical_format::CheckpointCertificateKind::ReleasedDrop,
        payload,
    )
    .unwrap();
    assert_eq!(
        worth_store_physical_format::decode_checkpoint_certificate(&frame).unwrap(),
        (
            worth_store_physical_format::CheckpointCertificateKind::ReleasedDrop,
            payload
        )
    );
    frame
}

pub(crate) fn assert_certificate_sequence_admitted(frames: &[Vec<u8>]) {
    let checkpoint = identity();
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(checkpoint));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(checkpoint));
    let binding = validate_binding(&BINDING, binding_scope(checkpoint, BINDING.len() as u64));
    let mut next_offset = FOOTER_OFFSET;
    let certificates: Vec<_> = frames
        .iter()
        .map(|frame| {
            let range = worth_store_physical_integrity::PhysicalByteRange::new(
                next_offset,
                frame.len() as u64,
            )
            .unwrap();
            next_offset = range.end_exclusive();
            (range, frame.as_slice())
        })
        .collect();
    let mut aggregate = worth_store_physical_format::CheckpointSelectiveRecordAggregate::new();
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
    let scope = worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(
        checkpoint,
        worth_store_physical_integrity::PhysicalByteRange::new(next_offset, footer.len() as u64)
            .unwrap(),
    );
    let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    )
    .with_certificates(&certificates);
    assert!(matches!(
        validate_checkpoint_footer(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&footer),
            scope,
            basis
        )
        .0,
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Intact(_)
    ));
    crate::checkpoint_validation::certificate_grammar::prepared_retention::assert_exact_prepared_retention(frames, &footer);
    crate::checkpoint_validation::certificate_grammar::bounded_scrub::assert_admitted(
        frames, &footer,
    );
}

pub(crate) fn assert_certificate_sequence_denied(frames: &[Vec<u8>]) {
    crate::checkpoint_validation::certificate_grammar::bounded_scrub::assert_denied(
        frames, &FOOTER,
    );
    let checkpoint = identity();
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(checkpoint));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(checkpoint));
    let binding = validate_binding(&BINDING, binding_scope(checkpoint, BINDING.len() as u64));
    let mut next_offset = FOOTER_OFFSET;
    let certificates: Vec<_> = frames
        .iter()
        .map(|frame| {
            let range = worth_store_physical_integrity::PhysicalByteRange::new(
                next_offset,
                frame.len() as u64,
            )
            .unwrap();
            next_offset = range.end_exclusive();
            (range, frame.as_slice())
        })
        .collect();
    let scope = worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(
        checkpoint,
        worth_store_physical_integrity::PhysicalByteRange::new(next_offset, FOOTER.len() as u64)
            .unwrap(),
    );
    let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    )
    .with_certificates(&certificates);
    let (validation, _) = validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    );
    let worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(rejection) =
        validation
    else {
        panic!("ambiguous release certificate roster unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}
