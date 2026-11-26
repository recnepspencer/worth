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

#[test]
fn cursor_refuses_mismatched_child_bounds_and_non_uniform_depth_and_oversized_pages_preserve_cache()
{
    use super::super::member_frame::{encode_member, ChunkMemberKind};
    use super::super::paged_directory::page::DirectoryChild;
    use super::super::paged_directory::DirectoryPage;
    use crate::storage::data::RelationalStorageUnavailableCause as Cause;

    let store = MemberStore::default();
    let put_page = |page: DirectoryPage<u64, u64>| {
        let member = encode_member(ChunkMemberKind::DirectoryPage, &page, 64 * 1024).unwrap();
        let locator = member.locator;
        store.replace(locator, member.bytes);
        locator
    };

    // Child claims [0, 10], but leaf only carries [0, 9].
    let leaf = put_page(DirectoryPage::Leaf(vec![(0, 100), (9, 109)]));
    let bad_bounds_root = put_page(DirectoryPage::Branch(vec![DirectoryChild {
        first: 0,
        last: 10,
        page: leaf,
    }]));
    let acc = accountant(64 * 1024, 64 * 1024, 4096);
    let cache = Arc::new(ResidentPageCache::new(
        Arc::clone(&acc),
        ResidencyPoolKind::Head,
    ));
    let bad_dir = PagedDirectory::<u64, u64>::open(
        Some(PagedDirectoryRoot {
            root_page: bad_bounds_root,
            entry_count: 2,
        }),
        Arc::clone(&cache),
    );
    assert_eq!(bad_dir.get(&store, 0), Err(Cause::Damaged));

    // Left child is a Leaf (depth 1), right child is Branch -> Leaf (depth 2).
    let left_leaf = put_page(DirectoryPage::Leaf(vec![(0, 1), (1, 2)]));
    let right_leaf = put_page(DirectoryPage::Leaf(vec![(10, 11), (11, 12)]));
    let right_branch = put_page(DirectoryPage::Branch(vec![DirectoryChild {
        first: 10,
        last: 11,
        page: right_leaf,
    }]));
    let ragged_root = put_page(DirectoryPage::Branch(vec![
        DirectoryChild {
            first: 0,
            last: 1,
            page: left_leaf,
        },
        DirectoryChild {
            first: 10,
            last: 11,
            page: right_branch,
        },
    ]));
    let ragged_dir = PagedDirectory::<u64, u64>::open(
        Some(PagedDirectoryRoot {
            root_page: ragged_root,
            entry_count: 4,
        }),
        Arc::clone(&cache),
    );
    let mut cursor = ragged_dir.range_from(&store, 0).unwrap();
    assert_eq!(cursor.next_entry(), Ok(Some((0, 1))));
    assert_eq!(cursor.next_entry(), Ok(Some((1, 2))));
    assert_eq!(cursor.next_entry(), Err(Cause::Damaged));

    // Oversized page (> pool ceiling) refuses without wiping already cached pages.
    let tiny_acc = accountant(512, 64 * 1024, 4096);
    let tiny_cache = ResidentPageCache::<DirectoryPage<u64, u64>>::new(
        Arc::clone(&tiny_acc),
        ResidencyPoolKind::Head,
    );
    let small_leaf = put_page(DirectoryPage::Leaf(vec![(1, 2)]));
    let huge_leaf = put_page(DirectoryPage::Leaf((0..128).map(|k| (k, k)).collect()));
    assert!(tiny_cache.page(&store, small_leaf).is_ok());
    assert_eq!(tiny_cache.observation().resident_pages, 1);
    assert_eq!(
        tiny_cache.page(&store, huge_leaf),
        Err(Cause::MemoryExhausted)
    );
    assert_eq!(
        tiny_cache.observation().resident_pages,
        1,
        "oversized page refusal must not evict live cached pages"
    );
    assert_eq!(tiny_cache.observation().evictions, 0);
}
