//! A paged directory reopens from its fixed-size root alone; point lookups
//! fault only their path and a full scan pages through a budget smaller than
//! the directory, at `1x` and `10x` entry counts alike (`G7`).
use std::sync::Arc;

use super::super::page_cache::ResidentPageCache;
use super::super::paged_directory::{
    DirectoryBuildError, PagedDirectory, PagedDirectoryBuilder, PagedDirectoryRoot,
};
use super::member_store::{accountant, MemberStore};
use crate::storage::residency::working_set::ResidencyPoolKind;

const PAGE_ENTRIES: usize = 16;
const BUDGET: u64 = 8 * 1024;

/// A reservation-table shaped directory: slot -> (generation, reserved).
fn build(store: &MemberStore, entries: u64) -> PagedDirectoryRoot {
    let mut builder =
        PagedDirectoryBuilder::<u64, (u32, bool)>::new(PAGE_ENTRIES, 64 * 1024).unwrap();
    for slot in 0..entries {
        builder
            .push(&mut &*store, slot * 3, ((slot % 7) as u32, slot % 2 == 0))
            .unwrap();
    }
    builder
        .finish(&mut &*store)
        .unwrap()
        .expect("non-empty directory")
}

#[test]
fn reopen_footprint_and_lookup_faults_stay_fixed_as_entries_grow_tenfold() {
    let store = MemberStore::default();
    let mut lookup_faults = Vec::new();
    for entries in [4_000_u64, 40_000] {
        let root = build(&store, entries);
        assert_eq!(root.entry_count, entries);
        let accountant = accountant(BUDGET, 1 << 20, 4096);
        let cache = Arc::new(ResidentPageCache::new(
            Arc::clone(&accountant),
            ResidencyPoolKind::Head,
        ));
        let directory = PagedDirectory::<u64, (u32, bool)>::open(Some(root), Arc::clone(&cache));
        assert_eq!(cache.observation().faults, 0, "reopen reads no page");
        assert_eq!(accountant.charged_bytes(ResidencyPoolKind::Head), 0);

        let probe = entries / 2;
        assert_eq!(
            directory.get(&store, probe * 3).unwrap(),
            Some(((probe % 7) as u32, probe % 2 == 0))
        );
        assert_eq!(directory.get(&store, probe * 3 + 1).unwrap(), None);
        lookup_faults.push(cache.observation().faults);

        let mut cursor = directory.range_from(&store, 0).unwrap();
        let mut seen = 0;
        while let Some((key, _)) = cursor.next_entry().unwrap() {
            assert_eq!(key, seen * 3);
            seen += 1;
            assert!(accountant.charged_bytes(ResidencyPoolKind::Head) <= BUDGET);
        }
        assert_eq!(seen, entries);
        assert!(
            cache.observation().evictions > 0,
            "the scan exceeded the budget and paged"
        );
    }
    // Depth grows by at most one level per 16x entries.
    assert!(
        lookup_faults[1] <= lookup_faults[0] + 1,
        "{lookup_faults:?}"
    );
}

#[test]
fn builder_refuses_unordered_keys_and_an_empty_directory_has_no_root() {
    let store = MemberStore::default();
    let mut builder = PagedDirectoryBuilder::<u64, u64>::new(PAGE_ENTRIES, 64 * 1024).unwrap();
    for key in 0..PAGE_ENTRIES as u64 * 2 {
        builder.push(&mut &store, key, key).unwrap();
    }
    assert_eq!(
        builder.push(&mut &store, 5, 5),
        Err(DirectoryBuildError::KeyOutOfOrder)
    );
    let empty = PagedDirectoryBuilder::<u64, u64>::new(PAGE_ENTRIES, 64 * 1024).unwrap();
    assert_eq!(empty.finish(&mut &store).unwrap(), None);
    assert_eq!(
        PagedDirectoryBuilder::<u64, u64>::new(1, 64 * 1024).err(),
        Some(DirectoryBuildError::PageTooSmall)
    );
}
