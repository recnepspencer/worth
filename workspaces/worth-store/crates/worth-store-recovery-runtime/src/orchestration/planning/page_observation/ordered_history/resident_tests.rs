//! A comparison the walk retains stops at the entries recovery admits and
//! at the scratch it has left, and says which of the two it met.

use super::super::test_inventory;
use super::*;
use crate::entry::PhysicalRecoveryLimitDimension::{self, ManifestEntries, StagingBytes};
use crate::orchestration::planning::page_observation::PageLimit;
use crate::orchestration::recovery_budget::{allowance_for_test, recovery_limit_for_test};

const SEGMENT_WIDTH: u64 = std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64;
const RECORD_WIDTH: u64 = std::mem::size_of::<PersistedRecordIdentity>() as u64;
const AMPLE: u64 = 1 << 20;

fn inventory(pages: u64) -> RecoverySelectedSourceInventory {
    test_inventory::inventory(4, pages, 0)
}

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

fn budget(admitted: u64) -> ManifestEntryBudget {
    ManifestEntryBudget::new(admitted, 0)
}

fn limit(dimension: PhysicalRecoveryLimitDimension, observed: u64, admitted: u64) -> WalkFailure {
    WalkFailure::Limit(PageLimit::Recovery(recovery_limit_for_test(
        dimension, observed, admitted,
    )))
}

/// The walk's `admitted` bytes of staging.
const fn staging(admitted: u64) -> RecoveryAllowance {
    allowance_for_test(StagingBytes, admitted)
}

/// The walk holds this much of its scratch outside each check.
const HELD: u64 = 100;

#[test]
fn a_segment_comparison_past_the_admitted_entries_is_that_limit() {
    let (larger, smaller) = (inventory(3), inventory(2));
    let (source, result, scratch) =
        segment_pair_bounded(&larger, &smaller, &mut budget(3), AMPLE, staging(AMPLE)).unwrap();
    assert_eq!((source.len(), result.len()), (3, 2));
    assert!(scratch >= 5 * SEGMENT_WIDTH);
    for (source, result) in [(&larger, &smaller), (&smaller, &larger)] {
        let mut budget = budget(2);
        let past = limit(ManifestEntries, 3, 2);
        assert_eq!(
            segment_pair_bounded(source, result, &mut budget, AMPLE, staging(AMPLE)).err(),
            Some(past),
        );
        assert_eq!(
            budget
                .refused()
                .map(PageLimit::Recovery)
                .map(WalkFailure::Limit),
            Some(past)
        );
    }
}

#[test]
fn a_segment_comparison_past_the_scratch_left_is_that_limit() {
    let (larger, smaller) = (inventory(3), inventory(2));
    // The source alone does not fit.
    let available = 3 * SEGMENT_WIDTH - 1;
    assert_eq!(
        segment_pair_bounded(
            &larger,
            &smaller,
            &mut budget(3),
            available,
            staging(available + HELD)
        )
        .err(),
        Some(limit(
            StagingBytes,
            3 * SEGMENT_WIDTH + HELD,
            available + HELD
        )),
    );
    // The source fits and leaves too little for the result.
    let available = 5 * SEGMENT_WIDTH - 1;
    assert_eq!(
        segment_pair_bounded(
            &larger,
            &smaller,
            &mut budget(3),
            available,
            staging(available + HELD)
        )
        .err(),
        Some(limit(
            StagingBytes,
            5 * SEGMENT_WIDTH + HELD,
            available + HELD
        )),
    );
}

#[test]
fn dropped_records_past_a_limit_name_it_and_a_repeat_is_unverified() {
    let (manifest, derived) = ([record(3), record(1)], [record(2)]);
    assert_eq!(
        dropped_bounded(&manifest, &derived, &mut budget(3), AMPLE, staging(AMPLE)),
        Ok(vec![record(1), record(2), record(3)]),
    );
    assert_eq!(
        dropped_bounded(&manifest, &derived, &mut budget(2), AMPLE, staging(AMPLE)),
        Err(limit(ManifestEntries, 3, 2)),
    );
    let available = 3 * RECORD_WIDTH - 1;
    assert_eq!(
        dropped_bounded(
            &manifest,
            &derived,
            &mut budget(3),
            available,
            staging(available + HELD)
        ),
        Err(limit(
            StagingBytes,
            3 * RECORD_WIDTH + HELD,
            available + HELD
        )),
    );
    assert_eq!(
        dropped_bounded(
            &manifest,
            &[record(3)],
            &mut budget(3),
            AMPLE,
            staging(AMPLE)
        ),
        Err(WalkFailure::Unverified),
    );
}
