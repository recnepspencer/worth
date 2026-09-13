use crate::{WalTopologyDenial, WalTopologyDenialKind};

use super::scan_fixtures::{generation, lsn, range, segment};

pub(super) fn assert_denial_has_no_context<T>(
    result: Result<T, WalTopologyDenial>,
    expected: WalTopologyDenialKind,
) {
    let denial = expect_denial(result);
    assert_eq!(denial.kind(), expected);
    assert_eq!(denial.segment_id(), None);
    assert_eq!(denial.expected_generation(), None);
    assert_eq!(denial.observed_generation(), None);
    assert_eq!(denial.previous_range(), None);
    assert_eq!(denial.observed_range(), None);
    assert_eq!(denial.missing_from(), None);
    assert_eq!(denial.missing_to(), None);
}

pub(super) fn assert_segment_denial<T>(
    result: Result<T, WalTopologyDenial>,
    expected: WalTopologyDenialKind,
    segment_id: u64,
) {
    let denial = expect_denial(result);
    assert_eq!(denial.kind(), expected);
    assert_eq!(denial.segment_id(), Some(segment(segment_id)));
}

pub(super) fn assert_generation_denial<T>(
    result: Result<T, WalTopologyDenial>,
    segment_id: u64,
    expected_generation: u64,
    observed_generation: u64,
) {
    let denial = expect_denial(result);
    assert_eq!(denial.kind(), WalTopologyDenialKind::WrongGeneration);
    assert_eq!(denial.segment_id(), Some(segment(segment_id)));
    assert_eq!(
        denial.expected_generation(),
        Some(generation(expected_generation))
    );
    assert_eq!(
        denial.observed_generation(),
        Some(generation(observed_generation))
    );
}

pub(super) fn assert_range_pair_denial<T>(
    result: Result<T, WalTopologyDenial>,
    expected: WalTopologyDenialKind,
    previous_range: (u64, u64),
    observed_range: (u64, u64),
) {
    let denial = expect_denial(result);
    assert_eq!(denial.kind(), expected);
    assert_eq!(
        denial.previous_range(),
        Some(range(previous_range.0, previous_range.1))
    );
    assert_eq!(
        denial.observed_range(),
        Some(range(observed_range.0, observed_range.1))
    );
}

pub(super) fn assert_gap_denial<T>(
    result: Result<T, WalTopologyDenial>,
    missing_from: u64,
    missing_to: u64,
) {
    let denial = expect_denial(result);
    assert_eq!(denial.kind(), WalTopologyDenialKind::Gap);
    assert_eq!(denial.missing_from(), Some(lsn(missing_from)));
    assert_eq!(denial.missing_to(), Some(lsn(missing_to)));
}

fn expect_denial<T>(result: Result<T, WalTopologyDenial>) -> WalTopologyDenial {
    match result {
        Ok(_) => panic!("expected topology denial"),
        Err(denial) => denial,
    }
}
