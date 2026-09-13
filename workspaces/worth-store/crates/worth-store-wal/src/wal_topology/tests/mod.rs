mod cursor_assertions;
mod denial_assertions;
mod scan_fixtures;

use cursor_assertions::{assert_cursor_topology, assert_ordering_proof, assert_same_cursor_order};
use denial_assertions::{
    assert_denial_has_no_context, assert_gap_denial, assert_generation_denial,
    assert_range_pair_denial, assert_segment_denial,
};
use scan_fixtures::{
    admit_cursor, generation, named_scan_orders, range, scan_record, stale_scan_record, try_range,
};

use crate::{WalSegmentGeneration, WalSegmentId, WalTopologyDenialKind};

#[test]
fn independent_segment_scans_produce_the_same_ordered_replay_cursor() {
    let mut admitted = named_scan_orders().into_iter().map(|(name, records)| {
        (
            name,
            admit_cursor(records)
                .unwrap_or_else(|denial| panic!("{name} scan should admit topology: {denial:?}")),
        )
    });
    let (_, canonical) = admitted.next().expect("canonical scan should exist");

    assert_cursor_topology(&canonical, &[(11, 0, 2), (12, 2, 4), (13, 4, 6)]);
    assert_ordering_proof(&canonical, 3, 3, 3, 2, 0, 6);

    for (name, cursor) in admitted {
        assert_same_cursor_order(&canonical, &cursor, name);
    }
}

#[test]
fn replay_cursor_keeps_wide_ranges_as_segment_topology() {
    let cursor = admit_cursor(vec![
        scan_record(11, generation(7), 0, 1_000_000),
        scan_record(12, generation(7), 1_000_000, 2_500_000),
        scan_record(13, generation(7), 2_500_000, 4_000_000),
    ])
    .unwrap();

    assert_eq!(cursor.segments().len(), 3);
    assert_cursor_topology(
        &cursor,
        &[
            (11, 0, 1_000_000),
            (12, 1_000_000, 2_500_000),
            (13, 2_500_000, 4_000_000),
        ],
    );
    assert_ordering_proof(&cursor, 3, 3, 3, 2, 0, 4_000_000);
}

#[test]
fn lsn_ranges_define_contiguity_at_half_open_boundaries() {
    let previous = range(0, 2);
    assert!(previous.is_contiguous_with(range(2, 4)));
    assert!(!previous.is_contiguous_with(range(3, 5)));
    assert!(!previous.is_contiguous_with(range(1, 3)));
}

#[test]
fn topology_denies_gaps_before_cursor_construction() {
    let current = generation(7);
    assert_gap_denial(
        admit_cursor(vec![
            scan_record(11, current, 0, 2),
            scan_record(12, current, 3, 5),
        ]),
        2,
        3,
    );
}

#[test]
fn topology_denies_noncontiguous_segment_identities() {
    let current = generation(7);
    assert_segment_denial(
        admit_cursor(vec![
            scan_record(11, current, 0, 2),
            scan_record(13, current, 2, 4),
        ]),
        WalTopologyDenialKind::NonContiguousSegment,
        13,
    );
}

#[test]
fn topology_denies_empty_candidate_sets() {
    assert_denial_has_no_context(
        admit_cursor(Vec::new()),
        WalTopologyDenialKind::EmptyTopology,
    );
}

#[test]
fn topology_denies_duplicate_segments() {
    let current = generation(7);
    assert_segment_denial(
        admit_cursor(vec![
            scan_record(11, current, 0, 2),
            scan_record(11, current, 2, 4),
        ]),
        WalTopologyDenialKind::DuplicateSegment,
        11,
    );
}

#[test]
fn topology_denies_duplicate_lsn_ranges() {
    let current = generation(7);
    assert_range_pair_denial(
        admit_cursor(vec![
            scan_record(11, current, 0, 2),
            scan_record(12, current, 0, 2),
        ]),
        WalTopologyDenialKind::DuplicateLsn,
        (0, 2),
        (0, 2),
    );
}

#[test]
fn topology_denies_overlapping_lsn_ranges() {
    let current = generation(7);
    assert_range_pair_denial(
        admit_cursor(vec![
            scan_record(11, current, 0, 3),
            scan_record(12, current, 2, 4),
        ]),
        WalTopologyDenialKind::OverlappingRange,
        (0, 3),
        (2, 4),
    );
}

#[test]
fn topology_denies_stale_and_wrong_generation_segments() {
    let current = generation(7);
    assert_segment_denial(
        admit_cursor(vec![stale_scan_record(11, current, 0, 2)]),
        WalTopologyDenialKind::StaleSegment,
        11,
    );
    assert_generation_denial(
        admit_cursor(vec![scan_record(12, generation(9), 0, 2)]),
        12,
        7,
        9,
    );
}

#[test]
fn topology_denies_invalid_identity_generation_and_range_construction() {
    assert_denial_has_no_context(
        WalSegmentId::new(0).map(|_| ()),
        WalTopologyDenialKind::EmptySegmentId,
    );
    assert_denial_has_no_context(
        WalSegmentGeneration::new(0).map(|_| ()),
        WalTopologyDenialKind::InvalidSegmentGeneration,
    );
    assert_denial_has_no_context(
        try_range(2, 2).map(|_| ()),
        WalTopologyDenialKind::EmptyRange,
    );
    assert_denial_has_no_context(
        try_range(3, 2).map(|_| ()),
        WalTopologyDenialKind::InvertedRange,
    );
}
