use super::*;
use cap_std::ambient_authority;
use std::sync::Arc;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

#[test]
fn identity_published_after_an_incomplete_root_snapshot_requires_a_fresh_scaffold_walk() {
    let parent = tempfile::tempdir().unwrap();
    std::fs::create_dir(parent.path().join("namespace")).unwrap();
    let root = Dir::open_ambient_dir(parent.path(), ambient_authority()).unwrap();
    // Capture the real root roles before the initializer completes its scaffold.
    let names = root
        .entries()
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(names, [std::ffi::OsString::from("namespace")]);
    let mut first_roles = vec![NamespaceEntryObservation::canonical(
        StoreNamespaceRelativeRole::NamespaceDirectory,
        NamespaceEntryType::Directory,
    )];
    std::fs::create_dir(parent.path().join("families")).unwrap();
    std::fs::create_dir(parent.path().join("staging")).unwrap();
    publish_identity(parent.path());
    let boundary = boundary();
    // The namespace part of that same walk now witnesses the published identity.
    observe_namespace_directory(&root, &boundary, &mut first_roles).unwrap();
    let mixed = classify_store_namespace(&NamespaceRootObservation::directory(first_roles));
    assert!(matches!(
        mixed,
        StoreNamespaceClassification::Damaged(
            NamespaceDamage::PublishedIdentityWithoutCompleteScaffold
        )
    ));
    assert!(matches!(
        complete_scaffold_snapshot(mixed, &root, &boundary).unwrap(),
        StoreNamespaceClassification::Initialized { .. }
    ));
}

#[test]
fn a_persistently_missing_scaffold_directory_remains_damage_after_reobservation() {
    let parent = tempfile::tempdir().unwrap();
    std::fs::create_dir(parent.path().join("namespace")).unwrap();
    std::fs::create_dir(parent.path().join("staging")).unwrap();
    publish_identity(parent.path());
    let root = Dir::open_ambient_dir(parent.path(), ambient_authority()).unwrap();
    assert!(matches!(
        classify_opened_root(&root, &boundary()).unwrap(),
        StoreNamespaceClassification::Damaged(
            NamespaceDamage::PublishedIdentityWithoutCompleteScaffold
        )
    ));
    assert!(!parent.path().join("families").exists());
}

fn publish_identity(root: &std::path::Path) {
    let proposed = ProposedStoreIdentity::from_nonzero_bytes([17; 16]).unwrap();
    let record = StoreNamespaceIdentityRecord::new(StoreNamespaceVersion::CURRENT, proposed);
    std::fs::write(root.join("namespace/mutation.lock"), []).unwrap();
    let staged = root.join("namespace/fixture-identity");
    std::fs::write(&staged, record.encode()).unwrap();
    std::fs::rename(staged, root.join("namespace/identity")).unwrap();
}

fn boundary() -> MediaFaultInterposer {
    MediaFaultInterposer::new(
        Default::default(),
        Arc::new(super::super::operation_counters::MediaCounterCells::default()),
    )
}
