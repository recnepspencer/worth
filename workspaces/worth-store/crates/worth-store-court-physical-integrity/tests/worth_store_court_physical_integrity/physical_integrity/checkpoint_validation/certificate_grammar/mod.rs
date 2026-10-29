#[allow(unused_imports)] use crate::checkpoint_validation::certificate_grammar::*;
use {worth_store_physical_format::encode_checkpoint_certificate, worth_store_physical_format::CheckpointCertificateKind, worth_store_physical_format::CheckpointSelectiveRecordAggregate, worth_store_physical_format::OriginalDropReservationRequestV1, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::ReleaseCheckpointAccumulatorV1, worth_store_physical_format::ReleaseCheckpointAccumulatorV2, worth_store_physical_format::ReleaseCheckpointBatchV1, worth_store_physical_format::ReleaseCheckpointNoReleaseV1, worth_store_physical_format::ReleasedDropWalFateWitnessV1};
use {worth_store_physical_integrity::validate_checkpoint_footer, worth_store_physical_integrity::CheckpointFooterIntegrityValidation, worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
use {crate::checkpoint_validation::support::assert_damage, crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_header, crate::checkpoint_validation::support::FOOTER_OFFSET};





#[test]
fn encoded_no_release_footer_is_admitted() {
    assert_certificate_sequence_admitted(&[no_release_frame()]);
}

#[test]
fn checksum_valid_no_release_under_another_checkpoint_is_denied() {
    let header = validate_header(&HEADER, header_scope_staged());
    let other = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        identity().store_identity(),
        std::num::NonZeroU64::new(8).unwrap(),
    );
    let marker = worth_store_physical_format::ReleaseCheckpointNoReleaseV1::new(
        other,
        header.source().root().generation(),
        [1; 32],
        0,
        [0; 32],
        [0; 32],
    )
    .unwrap();
    assert_certificate_sequence_denied(&[release_frame(&marker.encode())]);
}

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
    let batch = worth_store_physical_format::ReleaseCheckpointBatchV1::new(
        identity(),
        15,
        [1; 32],
        0,
        record(4),
        [2; 32],
        [3; 32],
        record(5),
        [4; 32],
        worth_store_physical_format::OriginalDropReservationRequestV1::new([5; 32], [6; 32], 10, 20).unwrap(),
        worth_store_physical_format::ReleasedDropWalFateWitnessV1::new(100, 110, [7; 32], [8; 32]).unwrap(),
        12,
        [9; 32],
        None,
        2,
        [10; 32],
        false,
    )
    .unwrap();
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
