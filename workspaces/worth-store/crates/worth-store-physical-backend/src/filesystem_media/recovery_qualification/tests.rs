use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

use super::{qualify_existing_recovery, RecoveryFilesystemQualificationError};
use crate::filesystem_media::{
    namespace_identity_admission, FilesystemMediaAdmissionAuthority, FilesystemMediaOwner,
};

#[test]
fn absent_and_incomplete_roots_are_refused_without_creation() {
    let parent = tempfile::tempdir().expect("test parent");
    let absent = parent.path().join("absent");
    let denial = qualify_existing_recovery(&absent)
        .err()
        .expect("absent root must be refused");
    assert_eq!(
        denial,
        RecoveryFilesystemQualificationError::ExistingStoreRequired
    );
    assert!(!absent.exists());

    let incomplete = parent.path().join("incomplete");
    std::fs::create_dir(&incomplete).expect("incomplete root");
    let denial = qualify_existing_recovery(&incomplete)
        .err()
        .expect("incomplete root must be refused");
    assert_eq!(
        denial,
        RecoveryFilesystemQualificationError::ExistingStoreRequired
    );
    assert_eq!(
        std::fs::read_dir(&incomplete)
            .expect("incomplete root remains")
            .count(),
        0
    );
}

#[test]
fn qualification_is_exclusive_and_persisted_identity_joins_afterward() {
    let parent = tempfile::tempdir().expect("test parent");
    let root = parent.path().join("store");
    let expected_store = initialize_store(&root);
    let qualified = qualify_existing_recovery(&root).expect("qualified existing store");
    let denial = qualify_existing_recovery(&root)
        .err()
        .expect("second recovery owner must contend");
    assert_eq!(
        denial,
        RecoveryFilesystemQualificationError::OwnershipContended
    );
    let admitted = qualified
        .admit_persisted_store()
        .expect("persisted identity admission");
    assert_eq!(admitted.store_identity, expected_store);
    let _ = admitted.owner.close();
}

#[test]
fn persisted_identity_is_read_after_the_exclusive_lease_is_acquired() {
    let parent = tempfile::tempdir().expect("test parent");
    let root = parent.path().join("store");
    let initial_store = initialize_store(&root);
    let qualified = qualify_existing_recovery(&root).expect("qualified existing store");
    let replacement =
        ProposedStoreIdentity::from_nonzero_bytes([93; 16]).expect("nonzero replacement identity");
    let replacement_record =
        StoreNamespaceIdentityRecord::new(StoreNamespaceVersion::CURRENT, replacement);

    std::fs::write(
        root.join("namespace").join("identity"),
        replacement_record.encode(),
    )
    .expect("adversarial identity replacement after lease acquisition");
    let admitted = qualified
        .admit_persisted_store()
        .expect("persisted identity admission");

    assert_ne!(admitted.store_identity, initial_store);
    assert_eq!(
        admitted.store_identity,
        replacement_record.published_identity()
    );
    let _ = admitted.owner.close();
}

#[test]
fn profile_is_stable_while_each_lease_has_a_new_media_generation() {
    let parent = tempfile::tempdir().expect("test parent");
    let root = parent.path().join("store");
    initialize_store(&root);
    let first = qualify_existing_recovery(&root).expect("first qualification");
    let first_profile = first.backend_profile().clone();
    let first_generation = first.media_generation();
    let _ = first.close();
    let second = qualify_existing_recovery(&root).expect("second qualification");
    assert_eq!(second.backend_profile(), &first_profile);
    assert_ne!(second.media_generation(), first_generation);
    let _ = second.close();
}

fn initialize_store(
    root: &std::path::Path,
) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
    let owner = FilesystemMediaOwner::admit(root, FilesystemMediaAdmissionAuthority::for_test())
        .expect("ordinary media owner");
    let identity = namespace_identity_admission::admit_store_identity(&owner)
        .expect("persisted identity")
        .stable_identity();
    let _ = owner.close();
    identity
}
