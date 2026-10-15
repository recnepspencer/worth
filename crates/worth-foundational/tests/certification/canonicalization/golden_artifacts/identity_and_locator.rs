use worth_foundational::{
    AspectLocator, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    BoundaryEpoch, BoundaryHandle, CanonicalDigestId, EquivalenceBasisId, LocatorAuthority,
};

use crate::certification::foundational_vocabulary::key;

#[test]
fn identity_and_locator_golden_keeps_equal_storage_meanings_distinct() {
    let artifact_id = BoundaryArtifactId::new(9);
    let artifact_locator = BoundaryArtifactLocator::new(artifact_id, BoundaryArtifactField::Basis);
    let aspect_locator = AspectLocator::new(LocatorAuthority::Authoritative, key("task.summary"));

    assert_eq!(artifact_locator.artifact_id(), BoundaryArtifactId::new(9));
    assert_eq!(artifact_locator.field(), BoundaryArtifactField::Basis);
    assert_eq!(aspect_locator.authority(), LocatorAuthority::Authoritative);
    assert_eq!(aspect_locator.aspect_key(), &key("task.summary"));
    let handle = BoundaryHandle::new(9);
    let epoch = BoundaryEpoch::new(9);
    assert_eq!(handle.get(), epoch.get());
    assert_eq!(EquivalenceBasisId::new(9).get(), 9);
    assert_eq!(CanonicalDigestId::new([7; 32]).bytes(), &[7; 32]);
}
