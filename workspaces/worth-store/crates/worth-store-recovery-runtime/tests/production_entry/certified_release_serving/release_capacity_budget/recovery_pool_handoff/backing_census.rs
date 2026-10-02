//! Actual native backing disposed when the real handoff is consumed into Core.

use worth_store::physical_runtime::PhysicalResidencyDimension as Dimension;
use worth_store_recovery_physics::{
    CheckpointCoveredWalArtifact, PhysicalWalFrameFacts, PhysicalWalSegmentCandidate,
};
use worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff;

pub(super) fn assert_disposed(
    boundary: &str,
    mut counters: impl FnMut(Dimension) -> (u64, u64, u64),
) {
    for dimension in [
        Dimension::MetadataBytes,
        Dimension::OperationBytes,
        Dimension::TotalBytes,
    ] {
        let (active, admitted, released) = counters(dimension);
        assert_eq!(active, 0, "{boundary}: {dimension:?}");
        assert_eq!(admitted, released);
    }
}

pub(super) fn retained_fingerprint_bytes(
    core: &worth_store::physical_runtime::RecoveredPhysicalRuntimeCore,
) -> u64 {
    let wal_bytes = core
        .selected_wal_owned_heap_bytes()
        .expect("actual independent WAL fingerprint");
    assert!(
        wal_bytes > 0,
        "genuine release media retains a nonempty fingerprint roster"
    );
    let head_bytes = core
        .selected_head_walk_owned_heap_bytes()
        .expect("actual independent full-tree head fingerprint");
    assert!(
        head_bytes > 0,
        "genuine released checkpoint retains a rooted head witness"
    );
    wal_bytes + head_bytes
}

pub(super) fn retained_evidence_bytes(handoff: &RecoveredPhysicalRuntimeHandoff) -> u64 {
    let tail = handoff.selected_sources().wal_tail();
    let bytes = (tail.segment_capacity() * std::mem::size_of::<PhysicalWalSegmentCandidate>()
        + tail.checkpoint_covered_capacity() * std::mem::size_of::<CheckpointCoveredWalArtifact>()
        + tail
            .segments()
            .iter()
            .map(|candidate| {
                candidate.frame_fact_capacity() * std::mem::size_of::<PhysicalWalFrameFacts>()
            })
            .sum::<usize>()) as u64;
    assert!(
        bytes > 0,
        "genuine release recovery carries actual WAL selection storage"
    );
    assert_eq!(handoff.wal_selection_charged_bytes(), bytes);
    let sample_bytes = handoff.freshness_sample().owned_heap_bytes().unwrap();
    assert!(
        sample_bytes > 0,
        "genuine release retains freshness evidence"
    );
    assert_eq!(handoff.freshness_sample().charged_bytes(), sample_bytes);
    bytes + sample_bytes
}
