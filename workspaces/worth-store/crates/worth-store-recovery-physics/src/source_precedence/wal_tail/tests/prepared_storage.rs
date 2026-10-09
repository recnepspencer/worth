//! Supplied storage stays unchanged while semantic tail policy remains exact.

use super::*;
use crate::source_precedence::CheckpointCoveredWalArtifact;

#[test]
fn supplied_frame_and_output_rosters_keep_pointer_capacity_and_spare_storage() {
    let verified = complete_two_frame_candidate(2);
    let inspection = verified.inspection();
    let mut frames = Vec::with_capacity(7);
    frames.extend_from_slice(verified.frame_facts());
    let frame_pointer = frames.as_ptr();
    let frame_capacity = frames.capacity();
    let candidate =
        PhysicalWalSegmentCandidate::from_frame_facts(inspection, None, frames).unwrap();
    assert_eq!(candidate.frame_facts().as_ptr(), frame_pointer);
    assert_eq!(candidate.frame_fact_capacity(), frame_capacity);
    let frame_bytes = (frame_capacity * std::mem::size_of::<PhysicalWalFrameFacts>()) as u64;
    assert_eq!(candidate.owned_heap_bytes(), Some(frame_bytes));

    let covered: Vec<CheckpointCoveredWalArtifact> = Vec::with_capacity(4);
    let covered_pointer = covered.as_ptr();
    let covered_capacity = covered.capacity();
    let retained = Vec::with_capacity(5);
    let retained_pointer = retained.as_ptr();
    let retained_capacity = retained.capacity();
    let selected = super::super::admit_physical_wal_tail(
        10,
        Some(5),
        vec![complete_candidate(1, 0, 10), candidate],
        covered,
        retained,
    )
    .unwrap();
    assert_eq!(selected.segments.as_ptr(), retained_pointer);
    assert_eq!(selected.segments.capacity(), retained_capacity);
    assert_eq!(selected.checkpoint_covered.as_ptr(), covered_pointer);
    assert_eq!(selected.checkpoint_covered.capacity(), covered_capacity);
    assert_eq!(selected.segments()[0].frame_fact_capacity(), frame_capacity);
    // Trimming changes the borrowed view, not the retained vector or its footprint.
    assert_eq!(
        selected.segments()[0].frame_facts().as_ptr(),
        frame_pointer.wrapping_add(1)
    );
    assert_eq!(selected.frame_count(), 1);
    let expected = frame_bytes
        + (retained_capacity * std::mem::size_of::<PhysicalWalSegmentCandidate>()) as u64
        + (covered_capacity * std::mem::size_of::<CheckpointCoveredWalArtifact>()) as u64;
    assert_eq!(selected.owned_heap_bytes(), Some(expected));
}

#[test]
fn insufficient_or_nonempty_prepared_rosters_are_typed_contract_denials() {
    assert_eq!(
        super::super::admit_physical_wal_tail(
            10,
            Some(10),
            vec![complete_candidate(1, 10, 20)],
            Vec::new(),
            Vec::with_capacity(1),
        ),
        Err(SelectedPhysicalWalTailDenial::PreparedStorage {
            required: 1,
            covered_capacity: 0,
            retained_capacity: 1,
            covered_length: 0,
            retained_length: 0,
        }),
    );
    assert_eq!(
        super::super::admit_physical_wal_tail(
            10,
            Some(10),
            vec![complete_candidate(1, 10, 20)],
            Vec::with_capacity(1),
            Vec::new(),
        ),
        Err(SelectedPhysicalWalTailDenial::PreparedStorage {
            required: 1,
            covered_capacity: 1,
            retained_capacity: 0,
            covered_length: 0,
            retained_length: 0,
        }),
    );
    let prior = select_candidates(10, Some(10), vec![complete_candidate(1, 0, 10)]).unwrap();
    let mut covered = Vec::with_capacity(1);
    covered.push(prior.checkpoint_covered()[0].clone());
    let mut retained = Vec::with_capacity(1);
    retained.push(complete_candidate(2, 10, 20));
    assert_eq!(
        super::super::admit_physical_wal_tail(
            10,
            Some(10),
            vec![complete_candidate(3, 20, 30)],
            covered,
            retained,
        ),
        Err(SelectedPhysicalWalTailDenial::PreparedStorage {
            required: 1,
            covered_capacity: 1,
            retained_capacity: 1,
            covered_length: 1,
            retained_length: 1,
        }),
    );
}

#[test]
fn duplicate_fully_covered_artifacts_deny_before_partitioning() {
    assert_eq!(
        select_candidates(
            10,
            Some(10),
            vec![complete_candidate(1, 0, 10), complete_candidate(1, 0, 10)]
        ),
        Err(SelectedPhysicalWalTailDenial::DuplicateArtifact),
    );
}

#[test]
fn checkpoint_frontier_inside_a_frame_is_not_a_valid_trim_boundary() {
    assert_eq!(
        select_candidates(5, Some(5), vec![complete_two_frame_candidate(1)]),
        Err(SelectedPhysicalWalTailDenial::CheckpointFrontierMismatch),
    );
}

#[test]
fn unrelated_higher_identity_covered_artifact_cannot_join_protected_suffix() {
    assert_eq!(
        select_candidates(
            20,
            Some(5),
            vec![
                complete_candidate(1, 0, 10),
                complete_candidate(2, 10, 20),
                complete_candidate(3, 20, 30),
                complete_candidate(9, 0, 5),
            ]
        ),
        Err(SelectedPhysicalWalTailDenial::CheckpointContinuationDiscontinuous),
    );
}

#[test]
fn replay_generation_mismatch_is_not_hidden_by_prepared_storage() {
    let other = WalSegmentArtifactIdentity::new(
        WalSegmentId::new(2).unwrap(),
        WalSegmentGeneration::new(2).unwrap(),
    );
    let plan = plan_frame(other, 20, 30, "other-generation", b"payload");
    let verified = inspect_verified_wal_segment(other, plan.frame().encoded_frame()).unwrap();
    assert_eq!(
        select_candidates(
            10,
            Some(10),
            vec![
                complete_candidate(1, 10, 20),
                candidate_from_verified(verified.to_owned_artifact(), None),
            ]
        ),
        Err(SelectedPhysicalWalTailDenial::GenerationMismatch),
    );
}
