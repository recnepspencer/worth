use worth_store_wal::{
    inspect_verified_wal_active_tail, inspect_verified_wal_segment, plan_wal_frame_append,
    LogSequenceNumber, WalAppendFrontier, WalLsnRange, WalSegmentArtifactIdentity,
    WalSegmentGeneration, WalSegmentId,
};

use super::*;

#[test]
fn generation_zero_without_checkpoint_or_wal_keeps_empty_origin() {
    let selected = admit_physical_wal_tail(0, None, Vec::new()).unwrap();
    assert!(selected.segments().is_empty());
    assert!(selected.protected_checkpoint_covered().is_empty());
    assert_eq!(selected.frame_count(), 0);
}

#[test]
fn semantic_segment_order_is_deterministic_and_checkpoint_contiguous() {
    let first = complete_candidate(1, 10, 20);
    let second = complete_candidate(2, 20, 30);
    let selected = admit_physical_wal_tail(10, Some(10), vec![second, first]).unwrap();
    assert_eq!(selected.segments()[0].identity().segment().get(), 1);
    assert_eq!(selected.segments()[1].identity().segment().get(), 2);
    assert_eq!(selected.frame_count(), 2);
}

#[test]
fn whole_checkpoint_covered_artifacts_are_retained_as_cleanup_facts() {
    let covered = complete_candidate(1, 0, 10);
    let retained = complete_candidate(2, 10, 20);
    let selected = admit_physical_wal_tail(10, Some(10), vec![retained, covered]).unwrap();

    assert_eq!(selected.segments().len(), 1);
    assert_eq!(selected.segments()[0].identity().segment().get(), 2);
    assert_eq!(selected.checkpoint_covered().len(), 1);
    assert_eq!(
        selected.checkpoint_covered()[0].identity().segment().get(),
        1
    );
    assert_eq!(
        selected.checkpoint_covered()[0].lsn_range().start().get(),
        0
    );
    assert_eq!(
        selected.checkpoint_covered()[0]
            .lsn_range()
            .end_exclusive()
            .get(),
        10
    );
    assert!(selected.checkpoint_covered()[0].byte_count() > 0);
    assert!(selected.checkpoint_covered()[0].cleanup_safe());
}

#[test]
fn interrupted_checkpoint_covered_artifact_remains_an_unsafe_cleanup_fact() {
    let covered = interrupted_candidate(1, 0, 10, 20);
    let observed = covered.interrupted_tail().unwrap().observed_bytes();
    let retained = complete_candidate(2, 10, 20);
    let selected = admit_physical_wal_tail(10, Some(10), vec![retained, covered]).unwrap();

    assert_eq!(selected.checkpoint_covered().len(), 1);
    assert!(!selected.checkpoint_covered()[0].cleanup_safe());
    assert_eq!(selected.checkpoint_covered()[0].byte_count(), observed);
}

#[test]
fn segment_and_lsn_gaps_are_independently_rejected() {
    assert_eq!(
        admit_physical_wal_tail(
            10,
            Some(10),
            vec![complete_candidate(1, 10, 20), complete_candidate(3, 20, 30)],
        ),
        Err(SelectedPhysicalWalTailDenial::SegmentGap),
        "MUTANT_PREDICATE:c8-wal-segment-gap-accepted"
    );
    assert_eq!(
        admit_physical_wal_tail(
            10,
            Some(10),
            vec![complete_candidate(1, 10, 20), complete_candidate(2, 21, 30)],
        ),
        Err(SelectedPhysicalWalTailDenial::LsnGap),
        "MUTANT_PREDICATE:c8-wal-lsn-gap-accepted"
    );
}

#[test]
fn interrupted_suffix_is_legal_only_on_the_terminal_segment() {
    let interrupted = interrupted_candidate(1, 10, 20, 30);
    let terminal = admit_physical_wal_tail(10, Some(10), vec![interrupted.clone()]).unwrap();
    assert!(terminal.segments()[0].interrupted_tail().is_some());
    assert_eq!(
        admit_physical_wal_tail(
            10,
            Some(10),
            vec![interrupted, complete_candidate(2, 20, 30)]
        ),
        Err(SelectedPhysicalWalTailDenial::InterruptedMiddleSegment),
        "MUTANT_PREDICATE:c8-interrupted-middle-wal-accepted"
    );
}

#[test]
fn checkpoint_only_keeps_exact_final_physical_continuation_anchor() {
    let selected = admit_physical_wal_tail(
        20,
        Some(20),
        vec![complete_candidate(1, 0, 10), complete_candidate(2, 10, 20)],
    )
    .unwrap();
    assert!(selected.segments().is_empty());
    assert_eq!(selected.frame_count(), 0);
    assert_eq!(selected.protected_checkpoint_covered().len(), 1);
    assert_eq!(
        selected.protected_checkpoint_covered()[0]
            .identity()
            .segment()
            .get(),
        2
    );
}

#[test]
fn fuzzy_checkpoint_keeps_exact_covered_chain_from_binding_cutoff() {
    let selected = admit_physical_wal_tail(
        20,
        Some(5),
        vec![
            complete_candidate(1, 0, 10),
            complete_candidate(2, 10, 20),
            complete_candidate(3, 20, 30),
        ],
    )
    .unwrap();
    assert_eq!(selected.frame_count(), 1);
    assert_eq!(selected.protected_checkpoint_covered().len(), 2);
    assert_eq!(
        selected.protected_checkpoint_covered()[0]
            .lsn_range()
            .start()
            .get(),
        0
    );
    assert_eq!(
        selected.protected_checkpoint_covered()[1]
            .lsn_range()
            .end_exclusive()
            .get(),
        20
    );
}

#[test]
fn physical_replay_segment_straddling_checkpoint_and_cutoff_needs_no_covered_anchor() {
    let selected =
        admit_physical_wal_tail(10, Some(5), vec![complete_two_frame_candidate(1)]).unwrap();
    assert_eq!(selected.frame_count(), 1);
    assert!(selected.protected_checkpoint_covered().is_empty());
}

#[test]
fn missing_ambiguous_and_interrupted_checkpoint_anchors_deny_selection() {
    assert_eq!(
        admit_physical_wal_tail(10, Some(10), vec![complete_candidate(1, 0, 9)]),
        Err(SelectedPhysicalWalTailDenial::CheckpointContinuationMissing)
    );
    assert_eq!(
        admit_physical_wal_tail(
            10,
            Some(10),
            vec![complete_candidate(1, 0, 10), complete_candidate(2, 5, 10)],
        ),
        Err(SelectedPhysicalWalTailDenial::CheckpointContinuationAmbiguous)
    );
    assert_eq!(
        admit_physical_wal_tail(10, Some(10), vec![interrupted_candidate(1, 0, 10, 20)]),
        Err(SelectedPhysicalWalTailDenial::CheckpointContinuationInterrupted)
    );
}

#[test]
fn protected_covered_chain_requires_exact_lsn_and_physical_identity_successors() {
    assert_eq!(
        admit_physical_wal_tail(
            20,
            Some(5),
            vec![complete_candidate(1, 0, 10), complete_candidate(3, 10, 20)],
        ),
        Err(SelectedPhysicalWalTailDenial::CheckpointContinuationDiscontinuous)
    );
    assert_eq!(
        admit_physical_wal_tail(
            20,
            Some(5),
            vec![complete_candidate(1, 0, 9), complete_candidate(2, 10, 20)],
        ),
        Err(SelectedPhysicalWalTailDenial::CheckpointContinuationMissing)
    );
}

fn complete_two_frame_candidate(segment: u64) -> PhysicalWalSegmentCandidate {
    let identity = identity(segment);
    let first = plan_frame(identity, 0, 10, "first-frame", b"first");
    let second = plan_wal_frame_append(
        first.resulting_frontier(),
        WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(20)).unwrap(),
        "second-frame",
        b"second",
    )
    .unwrap();
    let mut bytes = first.frame().encoded_frame().to_vec();
    bytes.extend_from_slice(second.frame().encoded_frame());
    let verified = inspect_verified_wal_segment(identity, &bytes).unwrap();
    candidate_from_verified(verified.to_owned_artifact(), None)
}

fn complete_candidate(segment: u64, start: u64, end: u64) -> PhysicalWalSegmentCandidate {
    let identity = identity(segment);
    let plan = plan_frame(identity, start, end, "test-frame", b"payload");
    let verified = inspect_verified_wal_segment(identity, plan.frame().encoded_frame()).unwrap();
    candidate_from_verified(verified.to_owned_artifact(), None)
}

fn interrupted_candidate(
    segment: u64,
    start: u64,
    first_end: u64,
    second_end: u64,
) -> PhysicalWalSegmentCandidate {
    let identity = identity(segment);
    let first = plan_frame(identity, start, first_end, "first-frame", b"first");
    let second = plan_wal_frame_append(
        first.resulting_frontier(),
        WalLsnRange::new(
            LogSequenceNumber::new(first_end),
            LogSequenceNumber::new(second_end),
        )
        .unwrap(),
        "second-frame",
        b"second",
    )
    .unwrap();
    let mut bytes = first.frame().encoded_frame().to_vec();
    bytes.extend_from_slice(&second.frame().encoded_frame()[..20]);
    let active = inspect_verified_wal_active_tail(identity, &bytes).unwrap();
    let interruption = active.interrupted_tail().map(|tail| {
        PhysicalWalInterruptionFacts::new(tail.valid_prefix_bytes(), tail.observed_bytes()).unwrap()
    });
    let verified = active.into_verified_prefix();
    candidate_from_verified(verified.to_owned_artifact(), interruption)
}

fn candidate_from_verified(
    verified: worth_store_wal::VerifiedWalArtifact,
    interruption: Option<PhysicalWalInterruptionFacts>,
) -> PhysicalWalSegmentCandidate {
    let inspection = verified.inspection();
    let facts = verified
        .frames()
        .iter()
        .map(|frame| PhysicalWalFrameFacts::new(frame.lsn_range(), frame.encoded_bytes()).unwrap())
        .collect();
    PhysicalWalSegmentCandidate::from_frame_facts(inspection, interruption, facts).unwrap()
}

fn identity(segment: u64) -> WalSegmentArtifactIdentity {
    WalSegmentArtifactIdentity::new(
        WalSegmentId::new(segment).unwrap(),
        WalSegmentGeneration::new(1).unwrap(),
    )
}

fn plan_frame(
    identity: WalSegmentArtifactIdentity,
    start: u64,
    end: u64,
    declared_identity: &str,
    payload: &[u8],
) -> worth_store_wal::PlannedWalFrameAppend {
    plan_wal_frame_append(
        WalAppendFrontier::empty(identity.segment(), identity.generation()),
        WalLsnRange::new(LogSequenceNumber::new(start), LogSequenceNumber::new(end)).unwrap(),
        declared_identity,
        payload,
    )
    .unwrap()
}
