use worth_foundational::{
    BoundaryArtifactId, BoundaryEpoch, BoundaryHandle, CanonicalDigestId, EquivalenceBasisId,
};

#[test]
fn equal_representations_keep_distinct_identity_meanings() {
    let artifact = BoundaryArtifactId::new(7);
    let handle = BoundaryHandle::new(7);
    let basis = EquivalenceBasisId::new(7);
    let epoch = BoundaryEpoch::new(7);
    let digest = CanonicalDigestId::new([7; 32]);

    assert_eq!(artifact.get(), 7);
    assert_eq!(handle.get(), 7);
    assert_eq!(basis.get(), 7);
    assert_eq!(epoch.get(), 7);
    assert_eq!(digest.bytes(), &[7; 32]);
}
