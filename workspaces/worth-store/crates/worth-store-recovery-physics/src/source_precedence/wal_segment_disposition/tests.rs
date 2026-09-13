use worth_store_wal::{
    LogSequenceNumber, WalLsnRange, WalSegmentArtifactIdentity, WalSegmentGeneration, WalSegmentId,
    WalSegmentInspection,
};

use super::*;

#[test]
fn empty_terminal_segment_remains_trailing_residue() {
    let disposition = classify_admitted_wal_segment(AdmittedWalSegmentPolicyInput::new(
        "segment-2-generation-1.wal".into(),
        identity(2),
        0,
        true,
        Some(AdmittedWalFrameRejectionKind::Truncated),
        None,
    ))
    .unwrap();
    let PhysicalWalSegmentDisposition::Residue {
        residue,
        torn_bytes,
    } = disposition
    else {
        panic!("empty terminal segment must remain residue")
    };
    assert_eq!(torn_bytes, 0);
    assert_eq!(
        residue.kind(),
        PhysicalRecoveryResidueKind::TrailingEmptyWalSegment
    );
}

#[test]
fn interrupted_start_is_legal_only_for_the_terminal_segment() {
    let terminal = classify_admitted_wal_segment(AdmittedWalSegmentPolicyInput::new(
        "segment-2-generation-1.wal".into(),
        identity(2),
        37,
        true,
        Some(AdmittedWalFrameRejectionKind::Truncated),
        None,
    ))
    .unwrap();
    assert!(matches!(
        terminal,
        PhysicalWalSegmentDisposition::Residue { torn_bytes: 37, .. }
    ));
    let middle = classify_admitted_wal_segment(AdmittedWalSegmentPolicyInput::new(
        "segment-1-generation-1.wal".into(),
        identity(1),
        37,
        false,
        Some(AdmittedWalFrameRejectionKind::Truncated),
        None,
    ))
    .unwrap();
    assert!(matches!(middle, PhysicalWalSegmentDisposition::Corrupt));
}

#[test]
fn exact_prefix_facts_produce_the_only_candidate_shape() {
    let identity = identity(1);
    let range = WalLsnRange::new(LogSequenceNumber::new(2), LogSequenceNumber::new(3)).unwrap();
    let inspection =
        WalSegmentInspection::from_admitted_frames(identity, range, 1, 128, [7; 32]).unwrap();
    let disposition = classify_admitted_wal_segment(AdmittedWalSegmentPolicyInput::new(
        identity.file_name(),
        identity,
        128,
        true,
        None,
        Some((
            inspection,
            vec![PhysicalWalFrameFacts::new(range, 128).unwrap()],
        )),
    ))
    .unwrap();
    let PhysicalWalSegmentDisposition::Candidate {
        candidate,
        torn_bytes,
    } = disposition
    else {
        panic!("complete admitted facts must produce a candidate")
    };
    assert_eq!(candidate.inspection(), inspection);
    assert_eq!(torn_bytes, 0);
}

fn identity(segment: u64) -> WalSegmentArtifactIdentity {
    WalSegmentArtifactIdentity::new(
        WalSegmentId::new(segment).unwrap(),
        WalSegmentGeneration::new(1).unwrap(),
    )
}
