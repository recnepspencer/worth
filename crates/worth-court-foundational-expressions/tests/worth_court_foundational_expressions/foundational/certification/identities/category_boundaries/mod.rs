use {worth_foundational::BoundaryArtifactId, worth_foundational::BoundaryEpoch, worth_foundational::BoundaryHandle, worth_foundational::CanonicalDigestId, worth_foundational::EquivalenceBasisId};

#[test]
fn equal_representations_keep_distinct_identity_meanings() {
    let artifact = worth_foundational::BoundaryArtifactId::new(7);
    let handle = worth_foundational::BoundaryHandle::new(7);
    let basis = worth_foundational::EquivalenceBasisId::new(7);
    let epoch = worth_foundational::BoundaryEpoch::new(7);
    let digest = worth_foundational::CanonicalDigestId::new([7; 32]);

    assert_eq!(artifact.get(), 7);
    assert_eq!(handle.get(), 7);
    assert_eq!(basis.get(), 7);
    assert_eq!(epoch.get(), 7);
    assert_eq!(digest.bytes(), &[7; 32]);
}
