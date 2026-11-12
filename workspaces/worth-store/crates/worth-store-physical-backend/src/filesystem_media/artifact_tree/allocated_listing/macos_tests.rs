use super::*;
use crate::filesystem_media::{
    ArtifactTreePathAllocationBoundary, ArtifactTreePathAllocator, ArtifactTreeStorageAllocator,
    FilesystemAccessPosture, FilesystemMediaOwner, FilesystemQualificationRequest,
    QualifiedFilesystemMedia,
};
use worth_proof::TransitionOutcome;

#[derive(Debug, PartialEq, Eq)]
struct Denied(Boundary);
#[derive(Default)]
struct Storage {
    deny: Option<Boundary>,
    last_retained: u64,
    roster_calls: usize,
    malformed_roster: bool,
    name_calls: usize,
    deny_name_call: Option<usize>,
}
impl ArtifactTreeStorageAllocator for Storage {
    type Denial = Denied;
}
impl ArtifactTreePathAllocator for Storage {
    type PathBacking = ();
    fn admit_path_backing(
        &mut self,
        _: ArtifactTreePathAllocationBoundary,
        _: u64,
    ) -> Result<(), Denied> {
        Ok(())
    }
}
impl ArtifactTreeListingAllocator for Storage {
    fn listing_storage_change(&mut self, change: Change) -> Result<(), Denied> {
        if let Change::Admit {
            boundary: Boundary::EntryName,
            ..
        } = change
        {
            self.name_calls += 1;
            if self.deny_name_call == Some(self.name_calls) {
                return Err(Denied(Boundary::EntryName));
            }
        }
        match change {
            Change::Admit { boundary, .. } if self.deny == Some(boundary) => {
                return Err(Denied(boundary))
            }
            Change::Settle { retained_bytes } => self.last_retained = retained_bytes,
            _ => {}
        }
        Ok(())
    }
    fn allocate_listing_roster(
        &mut self,
        count: usize,
    ) -> Result<Vec<ArtifactTreeDirectoryEntry>, Denied> {
        self.roster_calls += 1;
        Ok(Vec::with_capacity(
            count + usize::from(self.malformed_roster),
        ))
    }
}

fn qualified_media() -> (tempfile::TempDir, QualifiedFilesystemMedia) {
    let parent = tempfile::tempdir().unwrap();
    let result = FilesystemMediaOwner::qualify(FilesystemQualificationRequest::production(
        parent.path().join("store"),
        FilesystemAccessPosture::CoordinatedServiceAccount,
    ))
    .into_raw();
    let TransitionOutcome::Success(media) = result else {
        panic!("native media must qualify")
    };
    (parent, media)
}

#[test]
fn real_native_listing_preserves_names_types_and_independent_iteration_position() {
    use worth_store_physical_format::store_namespace::NamespaceEntryType;
    let (parent, owner) = qualified_media();
    let directory = ArtifactTreeDirectory::families();
    let tree = owner.artifact_tree();
    tree.write_new(&directory.file("actual-界.data").unwrap(), b"bytes")
        .unwrap();
    tree.create_directory(&directory.child("child").unwrap())
        .unwrap();
    std::os::unix::fs::symlink("actual-界.data", parent.path().join("store/families/link"))
        .unwrap();
    for _ in 0..2 {
        let mut storage = Storage::default();
        let entries = tree
            .list_bounded_with_allocator(&directory, 3, &mut storage)
            .unwrap();
        assert_eq!(entries.len(), 3);
        for (name, kind) in [
            ("actual-界.data", NamespaceEntryType::RegularFile),
            ("child", NamespaceEntryType::Directory),
            ("link", NamespaceEntryType::LinkLike),
        ] {
            assert!(entries
                .iter()
                .any(|entry| entry.name() == name && entry.entry_type() == kind));
        }
        let retained = entries.capacity() * std::mem::size_of::<ArtifactTreeDirectoryEntry>()
            + entries
                .into_iter()
                .map(|entry| entry.into_parts().0.capacity())
                .sum::<usize>();
        assert_eq!(storage.last_retained, retained as u64);
    }
    owner.close();
}

#[test]
fn provider_name_and_roster_denials_dispose_all_listing_storage() {
    let (_parent, owner) = qualified_media();
    let directory = ArtifactTreeDirectory::families();
    owner
        .artifact_tree()
        .write_new(&directory.file("present").unwrap(), b"bytes")
        .unwrap();
    for boundary in [
        Boundary::ProviderIterator,
        Boundary::EntryName,
        Boundary::EntryRoster,
    ] {
        let mut storage = Storage {
            deny: Some(boundary),
            ..Storage::default()
        };
        assert!(
            matches!(owner.artifact_tree().list_bounded_with_allocator(&directory, 1, &mut storage),
            Err(ArtifactTreeAllocatedListingFailure::Allocation { cause: Denied(observed), .. }) if observed == boundary)
        );
        assert_eq!(storage.last_retained, 0);
        assert_eq!(storage.roster_calls, 0);
    }
    let mut storage = Storage {
        malformed_roster: true,
        ..Storage::default()
    };
    assert!(matches!(
        owner
            .artifact_tree()
            .list_bounded_with_allocator(&directory, 1, &mut storage),
        Err(ArtifactTreeAllocatedListingFailure::BufferLengthMismatch { .. })
    ));
    assert_eq!(storage.last_retained, 0);
    owner.close();
}

#[test]
fn even_empty_listing_admits_the_iterator_attempt_before_observing_eof() {
    let (_parent, owner) = qualified_media();
    let mut storage = Storage {
        deny: Some(Boundary::EntryName),
        ..Storage::default()
    };
    assert!(matches!(
        owner.artifact_tree().list_bounded_with_allocator(
            &ArtifactTreeDirectory::families(),
            1,
            &mut storage
        ),
        Err(ArtifactTreeAllocatedListingFailure::Allocation {
            cause: Denied(Boundary::EntryName),
            ..
        })
    ));
    assert_eq!(storage.last_retained, 0);
    owner.close();
}

#[test]
fn extra_entry_attempt_requires_backing_before_cardinality_refusal() {
    let (_parent, media) = qualified_media();
    let directory = ArtifactTreeDirectory::families();
    let tree = media.artifact_tree();
    for name in ["first", "second"] {
        tree.write_new(&directory.file(name).unwrap(), b"bytes")
            .unwrap();
    }
    // First pre-next, first name copy, then the extra-entry pre-next grant.
    let mut storage = Storage {
        deny_name_call: Some(3),
        ..Storage::default()
    };
    assert!(matches!(
        tree.list_bounded_with_allocator(&directory, 1, &mut storage),
        Err(ArtifactTreeAllocatedListingFailure::Allocation {
            cause: Denied(Boundary::EntryName),
            ..
        })
    ));
    assert_eq!(storage.name_calls, 3);
    assert_eq!(storage.roster_calls, 1);
    assert_eq!(storage.last_retained, 0);
    let mut storage = Storage::default();
    assert!(
        matches!(tree.list_bounded_with_allocator(&directory, 1, &mut storage),
        Err(ArtifactTreeAllocatedListingFailure::Media(failure))
            if failure.kind() == ArtifactTreeFailureKind::AccessLimitExceeded
            && failure.access_limit() == Some(super::super::ArtifactTreeAccessLimit {
                observed: 2, admitted: 1
            }))
    );
    assert_eq!(storage.last_retained, 0);
    media.close();
}

#[test]
fn denied_pre_next_storage_prevents_native_cstring_allocation() {
    use crate::filesystem_media::tests::allocation_probe;
    if !allocation_probe::alone_in_its_process(
        module_path!(),
        "denied_pre_next_storage_prevents_native_cstring_allocation",
    ) {
        return;
    }
    let (_parent, owner) = qualified_media();
    let directory = ArtifactTreeDirectory::families();
    owner
        .artifact_tree()
        .write_new(&directory.file("a-real-native-entry").unwrap(), b"bytes")
        .unwrap();
    let mut storage = Storage {
        deny: Some(Boundary::EntryName),
        ..Storage::default()
    };
    let mut result = None;
    let allocated = allocation_probe::allocated_bytes_during(|| {
        result = Some(owner.artifact_tree().list_bounded_with_allocator(
            &directory,
            1,
            &mut storage,
        ));
    });
    assert!(matches!(
        result.unwrap(),
        Err(ArtifactTreeAllocatedListingFailure::Allocation {
            cause: Denied(Boundary::EntryName),
            ..
        })
    ));
    assert_eq!(
        allocated, 0,
        "denial must precede next() allocating its CString"
    );
    assert_eq!(storage.last_retained, 0);
    owner.close();
}
