use super::canonical_free_entries;
use worth_store_physical_format::{ExtentArenaId, ExtentArenaRange, RecordFreeSpaceManifestEntry};

fn arena(offset: u64, length: u64) -> RecordFreeSpaceManifestEntry {
    RecordFreeSpaceManifestEntry::arena_range(
        ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), offset, length).unwrap(),
        2,
    )
    .unwrap()
}

#[test]
fn one_arena_admits_distinct_disjoint_free_runs_by_full_key() {
    let mut entries = [arena(8192, 4096), arena(0, 4096)];
    assert!(canonical_free_entries(&mut entries));
    assert_eq!(entries[0].arena_free_range().unwrap().offset(), 0);
    assert_eq!(entries[1].arena_free_range().unwrap().offset(), 8192);
}

#[test]
fn one_arena_still_rejects_duplicate_overlapping_and_touching_runs() {
    for mut entries in [
        [arena(0, 4096), arena(0, 4096)],
        [arena(0, 8192), arena(4096, 4096)],
        [arena(0, 4096), arena(4096, 4096)],
    ] {
        assert!(!canonical_free_entries(&mut entries));
    }
}
