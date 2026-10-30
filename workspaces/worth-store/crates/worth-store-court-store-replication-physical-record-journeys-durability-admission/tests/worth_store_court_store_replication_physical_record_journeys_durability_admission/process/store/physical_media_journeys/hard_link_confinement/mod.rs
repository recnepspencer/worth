use {
    crate::physical_media_journeys::admit_runtime, crate::physical_media_journeys::media_admission,
};

#[test]
fn mutation_lock_hard_link_cannot_modify_an_outside_sentinel() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let initialized = match admit_runtime(&root)
        .try_admit_filesystem_media(media_admission())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("the specimen store must initialize"),
    };
    assert!(matches!(
        initialized.close(),
        worth_store::physical_runtime::MediaShutdownOutcome::Released(_)
    ));

    let sentinel = parent.path().join("outside-sentinel");
    let sentinel_bytes = b"outside bytes must survive";
    std::fs::write(&sentinel, sentinel_bytes).unwrap();
    let lock = root.join("namespace/mutation.lock");
    std::fs::remove_file(&lock).unwrap();
    std::fs::hard_link(&sentinel, &lock).unwrap();

    let outcome = admit_runtime(&root)
        .try_admit_filesystem_media(media_admission())
        .into_raw();
    let worth_proof::TransitionOutcome::Denied(denial) = outcome else {
        panic!("a multi-link mutation lock must deny before publication")
    };
    assert!(matches!(
        denial.reason(),
        worth_store_physical_backend::MediaQualificationDenial::OwnerPreEffect {
            denial: worth_store_physical_backend::FilesystemMediaOwnerAdmissionDenial::Ownership(
                worth_store_physical_backend::MutationOwnershipDenial::Confinement(confinement)
            ),
            ..
        } if confinement.kind() == worth_store_physical_backend::NamespaceConfinementDenialKind::MultipleLinks
    ));
    assert_eq!(std::fs::read(&sentinel).unwrap(), sentinel_bytes);
    assert_eq!(std::fs::read(&lock).unwrap(), sentinel_bytes);
    denial.into_runtime().abort();
}
