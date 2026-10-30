pub(crate) use {
    worth_store::physical_runtime::recovery_wal::LogSequenceNumber,
    worth_store::physical_runtime::recovery_wal::WalLsnRange,
};
pub(crate) use {
    worth_store_recovery_physics::admit_physical_wal_tail,
    worth_store_recovery_physics::select_physical_recovery_sources,
    worth_store_recovery_physics::PhysicalSourceSelectionDenial,
    worth_store_recovery_physics::PhysicalWalSegmentCandidate,
};
pub(crate) use {
    worth_store_test_support::harness::recovery::deterministic_checkpoint_plus_tail_source,
    worth_store_test_support::harness::recovery::wal_tail::selected_wal_tail,
};

/// Candidates are owned admission inputs; rebuild each untrimmed candidate
/// from its verified inspection and frame facts.
pub(crate) fn fresh(
    candidates: &[PhysicalWalSegmentCandidate],
) -> Vec<PhysicalWalSegmentCandidate> {
    candidates
        .iter()
        .map(|candidate| {
            worth_store_recovery_physics::PhysicalWalSegmentCandidate::from_frame_facts(
                candidate.inspection(),
                candidate.interrupted_tail(),
                candidate.frame_facts().to_vec(),
            )
            .expect("untrimmed fixture candidate")
        })
        .collect()
}
