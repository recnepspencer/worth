use std::num::{NonZeroU32, NonZeroU64};

use worth_store_physical_backend::ArtifactTreeDirectory;
use worth_store_wal::{
    LogSequenceNumber, WalAppendFrontier, WalLsnRange, WalSegmentArtifactIdentity,
    WalSegmentGeneration, WalSegmentId,
};

use super::{PhysicalWalRuntimeState, PhysicalWalSegmentInventory};

#[test]
fn appended_wal_never_widens_the_checkpoint_source_before_its_barrier() {
    let mut state = wal_state(1, 9);

    assert_eq!(state.checkpoint_source_range(), None);
    assert!(state.record_durable_barrier(1, 5));

    let source = state.checkpoint_source_range().unwrap();
    assert_eq!(source.admitted_begin_lsn(), 1);
    assert_eq!(source.covered_end_lsn_exclusive(), 5);
    assert_eq!(state.frontier.last_lsn_end().unwrap().get(), 9);
}

#[test]
fn exact_contiguous_barriers_advance_checkpoint_truth_without_skipping() {
    let mut state = wal_state(11, 19);

    assert!(state.record_durable_barrier(11, 14));
    assert!(state.record_durable_barrier(14, 19));

    let source = state.checkpoint_source_range().unwrap();
    assert_eq!(source.admitted_begin_lsn(), 11);
    assert_eq!(source.covered_end_lsn_exclusive(), 19);
}

#[test]
fn reclaimable_closed_prefix_waits_for_successor_segment_barrier() {
    let mut state = wal_state(1, 10);
    assert!(!state.has_reclaimable_closed_prefix());
    assert!(state.record_durable_barrier(1, 10));
    assert!(!state.has_reclaimable_closed_prefix());

    // Append into successor segment 2 before its durable barrier completes.
    let seg2 = WalSegmentId::new(2).unwrap();
    let generation = WalSegmentGeneration::new(1).unwrap();
    let range2 = WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(15)).unwrap();
    state
        .segments
        .record_completed_append(
            WalSegmentArtifactIdentity::new(seg2, generation),
            range2,
            128,
        )
        .unwrap();
    state.frontier = WalAppendFrontier::observed(seg2, generation, 128, LogSequenceNumber::new(15));
    // Durable boundary is still at segment 1's end, so segment 1 must stay retained.
    assert!(!state.has_reclaimable_closed_prefix());

    // Once segment 2's barrier completes, segment 1 becomes reclaimable.
    assert!(state.record_durable_barrier(10, 15));
    assert!(state.has_reclaimable_closed_prefix());

    // An unresolved retirement hold overlapping segment 1 suppresses the cadence signal.
    state.unresolved_retirement_spans.push((
        crate::physical_runtime::durability::RetiredArtifact::Segment {
            segment: 1,
            generation: 1,
        },
        5,
        6,
    ));
    assert!(!state.has_reclaimable_closed_prefix());
}

#[test]
fn discontinuous_or_overreaching_barriers_seal_checkpoint_authority() {
    for (start, end) in [(2, 7), (1, 10), (1, 1)] {
        let mut state = wal_state(1, 9);

        assert!(!state.record_durable_barrier(start, end));
        assert!(state.sealed);
        assert_eq!(state.checkpoint_source_range(), None);
        assert!(!state.record_durable_barrier(1, 9));
    }
}

fn wal_state(lsn_start: u64, lsn_end: u64) -> PhysicalWalRuntimeState {
    let segment = WalSegmentId::new(1).unwrap();
    let generation = WalSegmentGeneration::new(1).unwrap();
    let range = WalLsnRange::new(
        LogSequenceNumber::new(lsn_start),
        LogSequenceNumber::new(lsn_end),
    )
    .unwrap();
    let mut segments = PhysicalWalSegmentInventory::empty_for_runtime_test();
    segments
        .record_completed_append(
            WalSegmentArtifactIdentity::new(segment, generation),
            range,
            128,
        )
        .unwrap();
    PhysicalWalRuntimeState {
        record_format: worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
        copy_obligations: Vec::new(),
        reopened_checkpoint_cutoff: 0,
        retained_maintenance: Vec::new(),
        frontier: WalAppendFrontier::observed(
            segment,
            generation,
            128,
            LogSequenceNumber::new(lsn_end),
        ),
        durable_lsn_end: None,
        active_artifact: ArtifactTreeDirectory::families()
            .file("wal-segment-1")
            .unwrap(),
        policy: crate::physical_runtime::PhysicalWalPolicy::segmented(
            crate::physical_runtime::WalSegmentByteLimit::new(NonZeroU64::new(1_024).unwrap()),
            crate::physical_runtime::WalSegmentInventoryLimit::new(NonZeroU32::new(8).unwrap()),
        ),
        segment_count: 1,
        in_flight: false,
        sealed: false,
        appended_frames: 1,
        appended_bytes: 128,
        rotations: 0,
        reclaimed_segments: 0,
        reclaimed_bytes: 0,
        reopened_frames: 0,
        reopened_publication_groups: Vec::new(),
        reopened_release_metadata: Vec::new(),
        maintenance: None,
        awaiting_barrier: false,
        unresolved_retirement_spans: Vec::new(),
        reopened_bytes: 0,
        reopen_peak_buffer_bytes: 0,
        segments,
    }
}
