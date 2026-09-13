use super::{
    inspect_bounded_wal_active_tail_with_evidence, inspect_complete_wal_segment,
    inspect_interrupted_wal_segment_start, inspect_verified_wal_active_tail,
    WalActiveTailInspectionDenial, WalSegmentArtifactIdentity,
};
use crate::wal_frame_integrity_scope_identity;
use crate::{
    plan_wal_frame_append, LogSequenceNumber, WalAppendFrontier, WalLsnRange, WalSegmentGeneration,
    WalSegmentId,
};

fn identity() -> WalSegmentArtifactIdentity {
    WalSegmentArtifactIdentity::new(
        WalSegmentId::new(7).unwrap(),
        WalSegmentGeneration::new(3).unwrap(),
    )
}

#[test]
fn wal_owner_adapter_exposes_only_canonical_prevalidation_identity() {
    let identity = identity();
    let adapted = wal_frame_integrity_scope_identity(identity);
    assert_eq!(adapted.segment().get(), identity.segment().get());
    assert_eq!(adapted.generation().get(), identity.generation().get());
}

#[test]
fn frame_cardinality_is_denied_before_the_crossing_frame_is_retained() {
    let bytes = two_frames();
    let exact = inspect_bounded_wal_active_tail_with_evidence(identity(), &bytes, 2).unwrap();
    assert_eq!(exact.into_verified_prefix().frames().len(), 2);

    let failure = inspect_bounded_wal_active_tail_with_evidence(identity(), &bytes, 1).unwrap_err();
    assert_eq!(
        failure.denial(),
        WalActiveTailInspectionDenial::FrameLimitExceeded {
            observed: 2,
            admitted: 1,
        }
    );
    assert_eq!(failure.frames_scanned(), 2);
}

fn two_frames() -> Vec<u8> {
    let first_range =
        WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(11)).unwrap();
    let first = plan_wal_frame_append(
        WalAppendFrontier::empty(identity().segment(), identity().generation()),
        first_range,
        "first",
        b"payload-a",
    )
    .unwrap();
    let second_range =
        WalLsnRange::new(LogSequenceNumber::new(11), LogSequenceNumber::new(13)).unwrap();
    let second = plan_wal_frame_append(
        first.resulting_frontier(),
        second_range,
        "second",
        b"payload-b",
    )
    .unwrap();
    [
        first.frame().encoded_frame(),
        second.frame().encoded_frame(),
    ]
    .concat()
}

#[test]
fn canonical_name_and_complete_segment_reconstruct_exact_identity_and_range() {
    let identity = identity();
    assert_eq!(
        WalSegmentArtifactIdentity::parse(&identity.file_name()),
        Some(identity)
    );
    assert!(WalSegmentArtifactIdentity::parse("segment-07-generation-3.wal").is_none());

    let bytes = two_frames();
    let inspection = inspect_complete_wal_segment(identity, &bytes).unwrap();
    assert_eq!(inspection.identity(), identity);
    assert_eq!(inspection.frame_count(), 2);
    assert_eq!(inspection.byte_count(), bytes.len() as u64);
    assert_eq!(inspection.lsn_range().start().get(), 10);
    assert_eq!(inspection.lsn_range().end_exclusive().get(), 13);
}

#[test]
fn incomplete_or_identity_substituted_segment_is_rejected() {
    let bytes = two_frames();
    assert!(inspect_complete_wal_segment(identity(), &bytes[..bytes.len() - 1]).is_err());
    let substituted =
        WalSegmentArtifactIdentity::new(WalSegmentId::new(8).unwrap(), identity().generation());
    assert!(inspect_complete_wal_segment(substituted, &bytes).is_err());
}

#[test]
fn active_tail_admits_only_a_partial_frame_after_a_verified_prefix() {
    let bytes = two_frames();
    let first_bytes = inspect_complete_wal_segment(identity(), &bytes)
        .unwrap()
        .byte_count()
        - (116 + b"payload-b".len() + 32) as u64;
    let retained = first_bytes as usize + 37;
    let active = inspect_verified_wal_active_tail(identity(), &bytes[..retained]).unwrap();
    let interrupted = active.interrupted_tail().unwrap();
    assert_eq!(interrupted.valid_prefix_bytes(), first_bytes);
    assert_eq!(interrupted.observed_bytes(), retained as u64);
    let prefix = active.into_verified_prefix().inspection();
    assert_eq!(prefix.frame_count(), 1);
    assert_eq!(prefix.byte_count(), first_bytes);
}

#[test]
fn active_tail_rejects_partial_first_frame_and_complete_digest_damage() {
    let bytes = two_frames();
    assert!(inspect_verified_wal_active_tail(identity(), &bytes[..37]).is_err());
    let mut damaged = bytes;
    damaged[116] ^= 0xff;
    assert!(inspect_verified_wal_active_tail(identity(), &damaged).is_err());
}

#[test]
fn interrupted_segment_start_is_distinct_from_a_verified_segment() {
    let bytes = two_frames();
    let interrupted = inspect_interrupted_wal_segment_start(identity(), &bytes[..37]).unwrap();
    assert_eq!(interrupted.observed_bytes(), 37);
    assert!(inspect_interrupted_wal_segment_start(identity(), &[]).is_err());
    assert!(inspect_interrupted_wal_segment_start(identity(), &bytes).is_err());

    let mut damaged = bytes;
    damaged[116] ^= 0xff;
    assert!(inspect_interrupted_wal_segment_start(identity(), &damaged).is_err());
}
