use worth_store::physical_runtime::recovery_wal::{LogSequenceNumber, WalLsnRange};
use worth_store_recovery_physics::{
    admit_physical_wal_tail, select_physical_recovery_sources, PhysicalSourceSelectionDenial,
    PhysicalWalSegmentCandidate,
};
use worth_store_test_support::harness::recovery::{
    deterministic_checkpoint_plus_tail_source, wal_tail::selected_wal_tail,
};

#[test]
fn final_source_join_requires_the_exact_checkpoint_used_for_wal_admission() {
    let source = deterministic_checkpoint_plus_tail_source();
    let checkpoint = source.checkpoint().unwrap();
    let frontier = checkpoint.wal_tail_begin_lsn();
    let cutoff = checkpoint
        .checkpoint()
        .compaction_cutover()
        .wal_cutoff_lsn_exclusive();
    let replay = source.wal_tail().segments();
    let covered_tail = selected_wal_tail(
        WalLsnRange::new(
            LogSequenceNumber::new(cutoff - 10),
            LogSequenceNumber::new(frontier),
        )
        .unwrap(),
    );
    let covered = covered_tail.segments();

    // Every variant independently passes WAL admission. Only the public join
    // can detect that its carried basis differs from the selected checkpoint.
    for (admitted_frontier, admitted_cutoff, candidates, expected) in [
        (frontier, Some(cutoff), replay, None),
        (frontier, Some(cutoff), covered, None),
        (
            frontier,
            None,
            replay,
            Some(PhysicalSourceSelectionDenial::WalCheckpointBasisMismatch),
        ),
        (
            frontier,
            Some(cutoff - 1),
            covered,
            Some(PhysicalSourceSelectionDenial::WalCheckpointBasisMismatch),
        ),
        (
            frontier + 10,
            Some(cutoff),
            replay,
            Some(PhysicalSourceSelectionDenial::WalCheckpointBasisMismatch),
        ),
    ] {
        let tail = admit_physical_wal_tail(
            admitted_frontier,
            admitted_cutoff,
            fresh(candidates),
            Vec::with_capacity(candidates.len()),
            Vec::with_capacity(candidates.len()),
        );
        let tail = tail.expect("negative twin must reach the final source join");
        let selected = select_physical_recovery_sources(
            source.root().clone(),
            source.page_facts().clone(),
            source.retained_previous_page_facts().cloned(),
            Some(checkpoint.clone()),
            tail,
            source.compaction(),
            Vec::new(),
        );
        assert_eq!(selected.err(), expected);
    }
}

/// Candidates are owned admission inputs; rebuild each untrimmed candidate
/// from its verified inspection and frame facts.
fn fresh(candidates: &[PhysicalWalSegmentCandidate]) -> Vec<PhysicalWalSegmentCandidate> {
    candidates
        .iter()
        .map(|candidate| {
            PhysicalWalSegmentCandidate::from_frame_facts(
                candidate.inspection(),
                candidate.interrupted_tail(),
                candidate.frame_facts().to_vec(),
            )
            .expect("untrimmed fixture candidate")
        })
        .collect()
}
