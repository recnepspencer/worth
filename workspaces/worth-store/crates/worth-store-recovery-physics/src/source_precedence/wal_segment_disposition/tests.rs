use worth_store_wal::{
    LogSequenceNumber, WalLsnRange, WalSegmentArtifactIdentity, WalSegmentGeneration, WalSegmentId,
    WalSegmentInspection,
};

use super::*;

#[test]
fn empty_terminal_segment_remains_trailing_residue() {
    let disposition = classify_admitted_wal_segment(AdmittedWalSegmentPolicyInput::new(
        identity(2),
        0,
        true,
        Some(AdmittedWalFrameRejectionKind::Truncated),
        None,
    ))
    .unwrap();
    let PhysicalWalSegmentDisposition::Residue {
        kind,
        observed_bytes,
        torn_bytes,
    } = disposition
    else {
        panic!("empty terminal segment must remain residue")
    };
    assert_eq!(torn_bytes, 0);
    assert_eq!(kind, PhysicalRecoveryResidueKind::TrailingEmptyWalSegment);
    assert_eq!(observed_bytes, 0);
}

#[test]
fn interrupted_start_is_legal_only_for_the_terminal_segment() {
    let terminal = classify_admitted_wal_segment(AdmittedWalSegmentPolicyInput::new(
        identity(2),
        37,
        true,
        Some(AdmittedWalFrameRejectionKind::Truncated),
        None,
    ))
    .unwrap();
    assert!(matches!(
        terminal,
        PhysicalWalSegmentDisposition::Residue {
            kind: PhysicalRecoveryResidueKind::InterruptedWalSegmentStart,
            observed_bytes: 37,
            torn_bytes: 37,
        }
    ));
    let middle = classify_admitted_wal_segment(AdmittedWalSegmentPolicyInput::new(
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
        identity,
        128,
        true,
        None,
        Some(inspection),
    ))
    .unwrap();
    let PhysicalWalSegmentDisposition::Candidate {
        preparation,
        torn_bytes,
    } = disposition
    else {
        panic!("complete admitted facts must produce a candidate")
    };
    let mut frame_facts = Vec::with_capacity(8);
    frame_facts.push(PhysicalWalFrameFacts::new(range, 128).unwrap());
    let pointer = frame_facts.as_ptr();
    let capacity = frame_facts.capacity();
    let candidate = preparation.bind_frame_facts(frame_facts).unwrap();
    assert_eq!(candidate.inspection(), inspection);
    assert_eq!(candidate.frame_facts().as_ptr(), pointer);
    assert_eq!(candidate.frame_fact_capacity(), capacity);
    assert_eq!(
        candidate.owned_heap_bytes(),
        Some((capacity * std::mem::size_of::<PhysicalWalFrameFacts>()) as u64,)
    );
    assert_eq!(torn_bytes, 0);
}

#[test]
fn known_rejection_is_corrupt_without_binding_prefix_storage() {
    let range = WalLsnRange::new(LogSequenceNumber::new(2), LogSequenceNumber::new(3)).unwrap();
    let inspection =
        WalSegmentInspection::from_admitted_frames(identity(1), range, 1, 128, [7; 32]).unwrap();
    for (terminal, rejection) in [
        (true, AdmittedWalFrameRejectionKind::Other),
        (false, AdmittedWalFrameRejectionKind::Other),
        (false, AdmittedWalFrameRejectionKind::Truncated),
    ] {
        assert!(matches!(
            classify_admitted_wal_segment(AdmittedWalSegmentPolicyInput::new(
                identity(1),
                129,
                terminal,
                Some(rejection),
                Some(inspection),
            )),
            Some(PhysicalWalSegmentDisposition::Corrupt)
        ));
    }
}

#[test]
fn candidate_preparation_preserves_interruption_and_rejects_wrong_frame_facts() {
    let range = WalLsnRange::new(LogSequenceNumber::new(2), LogSequenceNumber::new(3)).unwrap();
    let inspection =
        WalSegmentInspection::from_admitted_frames(identity(1), range, 1, 128, [7; 32]).unwrap();
    for bytes in [127, 128] {
        let Some(PhysicalWalSegmentDisposition::Candidate {
            preparation,
            torn_bytes,
        }) = classify_admitted_wal_segment(AdmittedWalSegmentPolicyInput::new(
            identity(1),
            133,
            true,
            Some(AdmittedWalFrameRejectionKind::Truncated),
            Some(inspection),
        ))
        else {
            panic!("terminal truncation with an admitted prefix must remain eligible");
        };
        assert_eq!(torn_bytes, 5);
        let candidate =
            preparation.bind_frame_facts(vec![PhysicalWalFrameFacts::new(range, bytes).unwrap()]);
        if bytes == 127 {
            assert!(candidate.is_none());
        } else {
            let interruption = candidate.unwrap().interrupted_tail().unwrap();
            assert_eq!(interruption.valid_prefix_bytes(), 128);
            assert_eq!(interruption.observed_bytes(), 133);
        }
    }
}

fn identity(segment: u64) -> WalSegmentArtifactIdentity {
    WalSegmentArtifactIdentity::new(
        WalSegmentId::new(segment).unwrap(),
        WalSegmentGeneration::new(1).unwrap(),
    )
}
