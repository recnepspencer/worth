use worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key;

#[test]
fn identity_and_locator_golden_keeps_equal_storage_meanings_distinct() {
    let artifact_id = worth_foundational::BoundaryArtifactId::new(9);
    let artifact_locator = worth_foundational::BoundaryArtifactLocator::new(
        artifact_id,
        worth_foundational::BoundaryArtifactField::Basis,
    );
    let aspect_locator = worth_foundational::AspectLocator::new(
        worth_foundational::LocatorAuthority::Authoritative,
        key("task.summary"),
    );

    assert_eq!(
        artifact_locator.artifact_id(),
        worth_foundational::BoundaryArtifactId::new(9)
    );
    assert_eq!(
        artifact_locator.field(),
        worth_foundational::BoundaryArtifactField::Basis
    );
    assert_eq!(
        aspect_locator.authority(),
        worth_foundational::LocatorAuthority::Authoritative
    );
    assert_eq!(aspect_locator.aspect_key(), &key("task.summary"));
    let handle = worth_foundational::BoundaryHandle::new(9);
    let epoch = worth_foundational::BoundaryEpoch::new(9);
    assert_eq!(handle.get(), epoch.get());
    assert_eq!(worth_foundational::EquivalenceBasisId::new(9).get(), 9);
    assert_eq!(
        worth_foundational::CanonicalDigestId::new([7; 32]).bytes(),
        &[7; 32]
    );
}
