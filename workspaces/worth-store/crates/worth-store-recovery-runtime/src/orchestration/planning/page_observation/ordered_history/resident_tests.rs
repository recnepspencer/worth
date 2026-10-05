//! A comparison the walk retains stops at the entries recovery admits and
//! at the scratch it has left, and says which of the two it met.

use super::super::test_inventory;
use super::*;

const SEGMENT_WIDTH: u64 = std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64;
const RECORD_WIDTH: u64 = std::mem::size_of::<PersistedRecordIdentity>() as u64;
const AMPLE: u64 = 1 << 20;

fn inventory(pages: u64) -> RecoverySelectedSourceInventory {
    test_inventory::inventory(4, pages, 0)
}

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

#[test]
fn a_segment_comparison_past_the_admitted_entries_is_that_limit() {
    let (larger, smaller) = (inventory(3), inventory(2));
    let (source, result, scratch) = segment_pair_bounded(&larger, &smaller, 3, AMPLE).unwrap();
    assert_eq!((source.len(), result.len()), (3, 2));
    assert!(scratch >= 5 * SEGMENT_WIDTH);
    for (source, result) in [(&larger, &smaller), (&smaller, &larger)] {
        assert_eq!(
            segment_pair_bounded(source, result, 2, AMPLE).err(),
            Some(WalkFailure::ManifestEntryLimit),
        );
    }
}

#[test]
fn a_segment_comparison_past_the_scratch_left_is_that_limit() {
    let (larger, smaller) = (inventory(3), inventory(2));
    // The source alone does not fit.
    assert_eq!(
        segment_pair_bounded(&larger, &smaller, 3, 3 * SEGMENT_WIDTH - 1).err(),
        Some(WalkFailure::MORE_SCRATCH),
    );
    // The source fits and leaves too little for the result.
    assert_eq!(
        segment_pair_bounded(&larger, &smaller, 3, 5 * SEGMENT_WIDTH - 1).err(),
        Some(WalkFailure::MORE_SCRATCH),
    );
}

#[test]
fn dropped_records_past_a_limit_name_it_and_a_repeat_is_unverified() {
    let (manifest, derived) = ([record(3), record(1)], [record(2)]);
    assert_eq!(
        dropped_bounded(&manifest, &derived, 3, AMPLE),
        Ok(vec![record(1), record(2), record(3)]),
    );
    assert_eq!(
        dropped_bounded(&manifest, &derived, 2, AMPLE),
        Err(WalkFailure::ManifestEntryLimit),
    );
    assert_eq!(
        dropped_bounded(&manifest, &derived, 3, 3 * RECORD_WIDTH - 1),
        Err(WalkFailure::MORE_SCRATCH),
    );
    assert_eq!(
        dropped_bounded(&manifest, &[record(3)], 3, AMPLE),
        Err(WalkFailure::Unverified),
    );
}
