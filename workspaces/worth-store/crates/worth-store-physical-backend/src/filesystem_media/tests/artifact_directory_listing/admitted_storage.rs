use super::*;
use crate::filesystem_media::ArtifactTreeAllocatedListingFailure;

#[derive(Debug, PartialEq, Eq)]
struct CallbackDenied;

#[derive(Default)]
struct Storage {
    changes: Vec<ArtifactTreeListingStorageChange>,
    rosters: Vec<usize>,
    deny: Option<ArtifactTreeListingAllocationBoundary>,
    undersized: bool,
}

impl crate::filesystem_media::ArtifactTreeStorageAllocator for Storage {
    type Denial = CallbackDenied;
}

impl crate::filesystem_media::ArtifactTreePathAllocator for Storage {
    type PathBacking = ();
    fn admit_path_backing(
        &mut self,
        _: crate::filesystem_media::ArtifactTreePathAllocationBoundary,
        _: u64,
    ) -> Result<(), CallbackDenied> {
        Ok(())
    }
}

impl ArtifactTreeListingAllocator for Storage {
    fn listing_storage_change(
        &mut self,
        change: ArtifactTreeListingStorageChange,
    ) -> Result<(), CallbackDenied> {
        self.changes.push(change);
        if let ArtifactTreeListingStorageChange::Admit { boundary, .. } = change {
            if self.deny == Some(boundary) {
                self.deny = None;
                return Err(CallbackDenied);
            }
        }
        Ok(())
    }

    fn allocate_listing_roster(
        &mut self,
        count: usize,
    ) -> Result<Vec<ArtifactTreeDirectoryEntry>, CallbackDenied> {
        self.rosters.push(count);
        Ok(if self.undersized {
            Vec::new()
        } else {
            Vec::with_capacity(count)
        })
    }
}

#[cfg(windows)]
#[test]
fn staged_listing_grows_from_observed_entries_and_settles_exact_retained_capacity() {
    use ArtifactTreeListingStorageChange::{Admit, Settle};
    let parent = tempfile::tempdir().unwrap();
    let media = qualified(&parent.path().join("store"), MediaFaultSchedule::default());
    let directory = ArtifactTreeDirectory::families()
        .child("staged-listing")
        .unwrap();
    let tree = media.artifact_tree();
    tree.create_directory(&directory).unwrap();
    for index in 0..7 {
        tree.write_new(
            &directory.file(&format!("entry-{index}-界.wal")).unwrap(),
            b"payload",
        )
        .unwrap();
    }
    let mut storage = Storage::default();
    // The callback spy tests C.4's mechanical ordering, not native funding.
    let entries = tree
        .list_bounded_with_allocator(&directory, 65_536, &mut storage)
        .unwrap();
    assert_eq!(entries.len(), 7);
    assert_eq!(
        storage.rosters,
        [4, 8],
        "maximum cardinality is not reserved up front"
    );
    let slot_bytes = entries.capacity() * std::mem::size_of::<ArtifactTreeDirectoryEntry>();
    let names: usize = entries
        .into_iter()
        .map(|entry| entry.into_parts().0.capacity())
        .sum();
    assert_eq!(
        storage.changes.last(),
        Some(&Settle {
            retained_bytes: (slot_bytes + names) as u64
        })
    );
    // The handle-path conversion is covered by the caller's operation
    // envelope; the first admission is sized from the actual path.
    let Some(&Admit {
        boundary: ArtifactTreeListingAllocationBoundary::ProviderIterator,
        required_bytes: iterator_peak,
    }) = storage.changes.first()
    else {
        panic!("provider iterator storage is the first admission");
    };
    assert!(
        iterator_peak < 0x7fff,
        "small actual path must not use the maximum path envelope"
    );
    assert_eq!(
        storage
            .changes
            .iter()
            .filter(|change| matches!(
                change,
                Admit {
                    boundary: ArtifactTreeListingAllocationBoundary::EntryName,
                    ..
                }
            ))
            .count(),
        7
    );
    media.close();
}

#[cfg(windows)]
#[test]
fn each_staged_listing_denial_disposes_storage_and_same_owner_retries() {
    use ArtifactTreeListingAllocationBoundary::*;
    for boundary in [ProviderIterator, EntryRoster, EntryName] {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("store");
        let media = qualified(&root, MediaFaultSchedule::default());
        let directory = seed_directory(&media);
        let before = media.counters();
        let mut storage = Storage {
            deny: Some(boundary),
            ..Storage::default()
        };
        let failure = media
            .artifact_tree()
            .list_bounded_with_allocator(&directory, 2, &mut storage)
            .unwrap_err();
        assert!(
            matches!(failure, ArtifactTreeAllocatedListingFailure::Allocation {
            requested, cause: CallbackDenied,
        } if requested > 0)
        );
        assert!(storage.changes.iter().any(|change| matches!(change,
            ArtifactTreeListingStorageChange::Admit { boundary: observed, .. } if *observed == boundary
        )));
        assert_eq!(
            storage.changes.last(),
            Some(&ArtifactTreeListingStorageChange::Settle { retained_bytes: 0 })
        );
        assert_eq!(media.counters().listing_batches(), before.listing_batches());
        assert_eq!(media.counters().listing_entries(), before.listing_entries());
        assert_eq!(
            media
                .counters()
                .completed_operations_for(MediaOperationRole::ListDirectory),
            before.completed_operations_for(MediaOperationRole::ListDirectory)
        );
        if matches!(boundary, ProviderIterator | EntryRoster) {
            assert!(storage.rosters.is_empty());
        }
        let entries = media
            .artifact_tree()
            .list_bounded_with_allocator(&directory, 2, &mut storage)
            .unwrap();
        assert_eq!(listing_names(&entries), seeded_names());
        assert_eq!(
            std::fs::read(root.join("families/listing/alpha.wal")).unwrap(),
            b"alpha"
        );
        media.close();
    }
}

#[cfg(windows)]
#[test]
fn malformed_listing_roster_denies_before_name_copy_and_empty_listing_needs_no_roster() {
    let parent = tempfile::tempdir().unwrap();
    let media = qualified(&parent.path().join("store"), MediaFaultSchedule::default());
    let directory = seed_directory(&media);
    let mut storage = Storage {
        undersized: true,
        ..Storage::default()
    };
    let failure = media
        .artifact_tree()
        .list_bounded_with_allocator(&directory, 2, &mut storage)
        .unwrap_err();
    assert!(
        matches!(failure, ArtifactTreeAllocatedListingFailure::BufferLengthMismatch {
        requested, observed: 0,
    } if requested == 2 * std::mem::size_of::<ArtifactTreeDirectoryEntry>())
    );
    assert!(!storage.changes.iter().any(|change| matches!(
        change,
        ArtifactTreeListingStorageChange::Admit {
            boundary: ArtifactTreeListingAllocationBoundary::EntryName,
            ..
        }
    )));
    let empty = directory.child("empty").unwrap();
    media.artifact_tree().create_directory(&empty).unwrap();
    let mut storage = Storage::default();
    assert!(media
        .artifact_tree()
        .list_bounded_with_allocator(&empty, 65_536, &mut storage)
        .unwrap()
        .is_empty());
    assert!(storage.rosters.is_empty());
    assert_eq!(
        storage.changes.last(),
        Some(&ArtifactTreeListingStorageChange::Settle { retained_bytes: 0 })
    );
    media.close();
}

#[cfg(not(windows))]
#[test]
fn unqualified_allocated_listing_denies_without_callbacks_but_raw_listing_remains_available() {
    let parent = tempfile::tempdir().unwrap();
    let media = qualified(&parent.path().join("store"), MediaFaultSchedule::default());
    let directory = seed_directory(&media);
    let mut storage = Storage::default();
    let before = media.counters();
    assert!(
        matches!(media.artifact_tree().list_bounded_with_allocator(&directory, 2, &mut storage),
            Err(ArtifactTreeAllocatedListingFailure::Media(failure))
                if failure.kind() == ArtifactTreeFailureKind::AccessLimitExceeded
        )
    );
    assert!(storage.changes.is_empty());
    assert!(storage.rosters.is_empty());
    assert_eq!(media.counters(), before);
    assert_eq!(
        listing_names(&media.artifact_tree().list_bounded(&directory, 2).unwrap()),
        seeded_names()
    );
    media.close();
}
