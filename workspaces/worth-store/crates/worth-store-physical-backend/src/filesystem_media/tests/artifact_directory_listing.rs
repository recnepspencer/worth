use std::collections::BTreeMap;
use std::path::Path;

use super::super::*;
use worth_proof::TransitionOutcome;
use worth_store_physical_format::store_namespace::NamespaceEntryType;

#[cfg(feature = "recovery-runtime-owner")]
mod admitted_storage;

#[test]
fn empty_artifact_directory_is_listed_but_absent_child_never_reaches_listing() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media = qualified(&root, MediaFaultSchedule::default());
    let directory = ArtifactTreeDirectory::families()
        .child("empty-listing")
        .unwrap();
    let tree = media.artifact_tree();
    tree.create_directory(&directory).unwrap();

    let before = media.counters();
    let entries = tree.list_bounded(&directory, 1).unwrap();
    assert!(entries.is_empty());
    let after_empty = media.counters();
    assert_eq!(after_empty.listing_batches(), before.listing_batches() + 1);
    assert_eq!(after_empty.listing_entries(), before.listing_entries());
    assert_eq!(
        after_empty.completed_operations_for(MediaOperationRole::ListDirectory),
        before.completed_operations_for(MediaOperationRole::ListDirectory) + 1
    );

    let missing = directory.child("missing").unwrap();
    let failure = tree.list_bounded(&missing, 1).unwrap_err();
    assert_eq!(failure.kind(), ArtifactTreeFailureKind::Absent);
    assert_eq!(failure.io_kind(), Some(std::io::ErrorKind::NotFound));
    let after_missing = media.counters();
    assert_eq!(
        after_missing.attempts_for(MediaOperationRole::ListDirectory),
        after_empty.attempts_for(MediaOperationRole::ListDirectory)
    );
    assert_eq!(
        after_missing.denied_before_effect_for(MediaOperationRole::OpenDirectory),
        after_empty.denied_before_effect_for(MediaOperationRole::OpenDirectory) + 1
    );
    assert_eq!(
        after_missing.listing_batches(),
        after_empty.listing_batches()
    );
    assert!(tree.list_bounded(&directory, 1).unwrap().is_empty());
    assert_eq!(
        std::fs::read_dir(root.join("families/empty-listing"))
            .unwrap()
            .count(),
        0
    );
    media.close();
}

#[test]
fn artifact_listing_accepts_exact_entry_limit_and_rejects_one_over() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media = qualified(&root, MediaFaultSchedule::default());
    let directory = seed_directory(&media);
    let tree = media.artifact_tree();

    let entries = tree.list_bounded(&directory, 2).unwrap();
    assert_eq!(listing_names(&entries), seeded_names());
    let before = media.counters();
    let failure = tree.list_bounded(&directory, 1).unwrap_err();
    assert_eq!(failure.kind(), ArtifactTreeFailureKind::AccessLimitExceeded);
    assert_eq!(
        failure.access_limit(),
        Some(ArtifactTreeAccessLimit {
            observed: 2,
            admitted: 1,
        })
    );
    let after = media.counters();
    assert_eq!(after.listing_batches(), before.listing_batches());
    assert_eq!(after.listing_entries(), before.listing_entries());
    assert_eq!(
        after.denied_before_effect_for(MediaOperationRole::ListDirectory),
        before.denied_before_effect_for(MediaOperationRole::ListDirectory) + 1
    );
    assert_eq!(
        listing_names(&tree.list_bounded(&directory, 2).unwrap()),
        seeded_names()
    );
    assert_eq!(
        std::fs::read(root.join("families/listing/alpha.wal")).unwrap(),
        b"alpha"
    );

    let before_zero = media.counters();
    assert_eq!(
        tree.list_bounded(&directory, 0).unwrap_err().kind(),
        ArtifactTreeFailureKind::AccessLimitExceeded
    );
    assert_eq!(media.counters(), before_zero);
    media.close();
}

#[test]
fn injected_artifact_listing_denial_keeps_entries_intact_and_same_owner_retries() {
    let parent = tempfile::tempdir().unwrap();
    let baseline = qualified(
        &parent.path().join("baseline"),
        MediaFaultSchedule::default(),
    );
    seed_directory(&baseline);
    let ordinal = baseline
        .counters()
        .attempts_for(MediaOperationRole::ListDirectory)
        + 1;
    baseline.close();

    let schedule = MediaFaultSchedule::for_certification(vec![MediaFaultRule::for_certification(
        MediaOperationRole::ListDirectory,
        ordinal,
        MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::PermissionDenied,
            raw_os_error: None,
        },
    )])
    .unwrap();
    let root = parent.path().join("faulted");
    let media = qualified(&root, schedule);
    let directory = seed_directory(&media);
    let before = media.counters();
    assert_eq!(
        before.attempts_for(MediaOperationRole::ListDirectory) + 1,
        ordinal
    );

    let failure = media
        .artifact_tree()
        .list_bounded(&directory, 2)
        .unwrap_err();
    assert_eq!(failure.kind(), ArtifactTreeFailureKind::DeniedBeforeEffect);
    assert_eq!(
        failure.io_kind(),
        Some(std::io::ErrorKind::PermissionDenied)
    );
    let denied = media.counters();
    assert_eq!(denied.fault_matches(), before.fault_matches() + 1);
    assert_eq!(
        denied.attempts_for(MediaOperationRole::ListDirectory),
        before.attempts_for(MediaOperationRole::ListDirectory) + 1
    );
    assert_eq!(
        denied.denied_before_effect_for(MediaOperationRole::ListDirectory),
        before.denied_before_effect_for(MediaOperationRole::ListDirectory) + 1
    );
    assert_eq!(
        denied.completed_operations_for(MediaOperationRole::ListDirectory),
        before.completed_operations_for(MediaOperationRole::ListDirectory)
    );
    assert_eq!(denied.listing_batches(), before.listing_batches());
    assert_eq!(denied.listing_entries(), before.listing_entries());
    assert_eq!(
        denied.explicit_heap_allocation_events(),
        before.explicit_heap_allocation_events()
    );
    assert_eq!(denied.completed_bytes(), before.completed_bytes());
    assert_eq!(
        std::fs::read(root.join("families/listing/alpha.wal")).unwrap(),
        b"alpha"
    );

    let entries = media.artifact_tree().list_bounded(&directory, 2).unwrap();
    assert_eq!(listing_names(&entries), seeded_names());
    let retried = media.counters();
    assert_eq!(retried.fault_matches(), denied.fault_matches());
    assert_eq!(retried.listing_batches(), denied.listing_batches() + 1);
    assert_eq!(retried.listing_entries(), denied.listing_entries() + 2);
    assert_eq!(
        retried.completed_operations_for(MediaOperationRole::ListDirectory),
        denied.completed_operations_for(MediaOperationRole::ListDirectory) + 1
    );
    assert!(retried.is_conserved());
    media.close();
}

#[cfg(windows)]
#[test]
fn long_local_artifact_directory_preserves_unicode_and_near_maximum_names() {
    use std::os::windows::ffi::OsStrExt;

    if !super::allocation_probe::alone_in_its_process(
        module_path!(),
        "long_local_artifact_directory_preserves_unicode_and_near_maximum_names",
    ) {
        return;
    }
    let parent = tempfile::tempdir().unwrap();
    let mut root = parent.path().to_path_buf();
    for component in ["a", "b", "c", "d", "e"] {
        root.push(format!("{component}-{}", "long".repeat(15)));
    }
    std::fs::create_dir_all(&root).expect("create genuine long local root");
    assert!(root.as_os_str().encode_wide().count() > 260);
    let media = qualified(&root, MediaFaultSchedule::default());
    let directory = ArtifactTreeDirectory::families().child("listing").unwrap();
    let tree = media.artifact_tree();
    tree.create_directory(&directory).unwrap();
    let near_maximum = format!("{}.frame", "界".repeat(240));
    let unicode = "report-🦀.wal";
    assert_eq!(near_maximum.encode_utf16().count(), 246);
    tree.write_new(&directory.file(&near_maximum).unwrap(), b"wide")
        .unwrap();
    tree.write_new(&directory.file(unicode).unwrap(), b"unicode")
        .unwrap();
    tree.create_directory(&directory.child("nested").unwrap())
        .unwrap();

    let mut requirement = None;
    let allocated = super::allocation_probe::allocated_bytes_during(|| {
        requirement = tree.listing_storage_requirement(3);
    });
    assert_eq!(allocated, 0, "listing storage requirement query allocated");
    let requirement =
        requirement.expect("current qualified Windows toolchain has a listing storage contract");
    assert!(tree.listing_storage_requirement(0).is_none());
    assert!(tree.listing_storage_requirement(usize::MAX).is_none());
    let before = media.counters();
    let entries = tree.list_bounded(&directory, 3).unwrap();
    assert_eq!(
        listing_names(&entries),
        BTreeMap::from([
            (near_maximum.clone(), NamespaceEntryType::RegularFile),
            (unicode.to_owned(), NamespaceEntryType::RegularFile),
            ("nested".to_owned(), NamespaceEntryType::Directory),
        ])
    );
    let after = media.counters();
    assert_eq!(after.listing_batches(), before.listing_batches() + 1);
    assert_eq!(after.listing_entries(), before.listing_entries() + 3);
    let slot_bytes = entries.capacity() * std::mem::size_of_val(&entries[0]);
    let name_bytes: usize = entries
        .into_iter()
        .map(|entry| entry.into_parts().0.capacity())
        .sum();
    let retained_bytes = u64::try_from(slot_bytes + name_bytes).unwrap();
    assert!(
        retained_bytes <= requirement.listing_retained_bytes(),
        "actual listing backing {retained_bytes} exceeds admitted retained bound {}",
        requirement.listing_retained_bytes()
    );
    assert_eq!(
        std::fs::read(root.join("families/listing").join(&near_maximum)).unwrap(),
        b"wide"
    );
    assert_eq!(
        std::fs::read(root.join("families/listing").join(unicode)).unwrap(),
        b"unicode"
    );
    media.close();
}

#[cfg(any(unix, windows))]
#[test]
fn artifact_listing_classifies_directory_link_but_never_enumerates_its_target() {
    let parent = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("outside.wal"), b"outside").unwrap();
    let root = parent.path().join("store");
    let media = qualified(&root, MediaFaultSchedule::default());
    let directory = ArtifactTreeDirectory::families().child("listing").unwrap();
    media.artifact_tree().create_directory(&directory).unwrap();
    let link = root.join("families/listing/linked");
    create_directory_link(outside.path(), &link).expect("create real directory-link fixture");

    let entries = media.artifact_tree().list_bounded(&directory, 1).unwrap();
    assert_eq!(
        listing_names(&entries),
        BTreeMap::from([("linked".to_owned(), NamespaceEntryType::LinkLike)])
    );
    let before = media.counters();
    let linked_directory = directory.child("linked").unwrap();
    let failure = media
        .artifact_tree()
        .list_bounded(&linked_directory, 1)
        .unwrap_err();
    assert_eq!(failure.kind(), ArtifactTreeFailureKind::DeniedBeforeEffect);
    let after = media.counters();
    assert_eq!(
        after.attempts_for(MediaOperationRole::ListDirectory),
        before.attempts_for(MediaOperationRole::ListDirectory)
    );
    assert_eq!(
        after.denied_before_effect_for(MediaOperationRole::OpenDirectory),
        before.denied_before_effect_for(MediaOperationRole::OpenDirectory) + 1
    );
    assert_eq!(after.listing_entries(), before.listing_entries());
    assert_eq!(
        std::fs::read(outside.path().join("outside.wal")).unwrap(),
        b"outside"
    );
    media.close();
}

fn qualified(root: &Path, schedule: MediaFaultSchedule) -> QualifiedFilesystemMedia {
    let request = FilesystemQualificationRequest::certification(
        root,
        FilesystemAccessPosture::CoordinatedServiceAccount,
    )
    .with_fault_schedule(schedule);
    match FilesystemMediaOwner::qualify(request).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("qualification failed"),
    }
}

fn seed_directory(media: &QualifiedFilesystemMedia) -> ArtifactTreeDirectory {
    let directory = ArtifactTreeDirectory::families().child("listing").unwrap();
    let tree = media.artifact_tree();
    tree.create_directory(&directory).unwrap();
    tree.write_new(&directory.file("alpha.wal").unwrap(), b"alpha")
        .unwrap();
    tree.create_directory(&directory.child("nested").unwrap())
        .unwrap();
    directory
}

fn seeded_names() -> BTreeMap<String, NamespaceEntryType> {
    BTreeMap::from([
        ("alpha.wal".to_owned(), NamespaceEntryType::RegularFile),
        ("nested".to_owned(), NamespaceEntryType::Directory),
    ])
}

fn listing_names(entries: &[ArtifactTreeDirectoryEntry]) -> BTreeMap<String, NamespaceEntryType> {
    let names: BTreeMap<_, _> = entries
        .iter()
        .map(|entry| {
            (
                entry.name().to_str().unwrap().to_owned(),
                entry.entry_type(),
            )
        })
        .collect();
    assert_eq!(entries.len(), names.len(), "listing repeated a source name");
    names
}

#[cfg(unix)]
fn create_directory_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}
