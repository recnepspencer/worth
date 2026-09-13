use crate::ReplayCursor;

use super::scan_fixtures::{generation, lsn};

pub(super) fn assert_cursor_topology(cursor: &ReplayCursor, expected: &[(u64, u64, u64)]) {
    assert_eq!(cursor_ranges(cursor), expected);
}

pub(super) fn assert_ordering_proof(
    cursor: &ReplayCursor,
    candidate_count: usize,
    accepted_segment_count: usize,
    ordered_range_count: usize,
    range_adjacency_check_count: usize,
    first_lsn: u64,
    end_lsn: u64,
) {
    let proof = cursor.ordering_proof();
    assert_eq!(proof.expected_generation(), generation(7));
    assert_eq!(proof.candidate_count(), candidate_count);
    assert_eq!(proof.accepted_segment_count(), accepted_segment_count);
    assert_eq!(proof.ordered_range_count(), ordered_range_count);
    assert_eq!(
        proof.range_adjacency_check_count(),
        range_adjacency_check_count
    );
    assert_eq!(proof.first_lsn(), lsn(first_lsn));
    assert_eq!(proof.end_lsn(), lsn(end_lsn));
    assert_eq!(cursor.first_lsn(), lsn(first_lsn));
    assert_eq!(cursor.end_lsn(), lsn(end_lsn));
}

pub(super) fn assert_same_cursor_order(
    left: &ReplayCursor,
    right: &ReplayCursor,
    order_name: &str,
) {
    assert_eq!(
        cursor_ranges(left),
        cursor_ranges(right),
        "{order_name} changed replay cursor order"
    );
    assert_eq!(
        left.ordering_proof(),
        right.ordering_proof(),
        "{order_name} changed ordering proof"
    );
}

fn cursor_ranges(cursor: &ReplayCursor) -> Vec<(u64, u64, u64)> {
    cursor
        .segments()
        .iter()
        .map(|entry| {
            (
                entry.segment_id().get(),
                entry.lsn_range().start().get(),
                entry.lsn_range().end_exclusive().get(),
            )
        })
        .collect()
}
