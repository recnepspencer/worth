#[test]
fn root_exports_are_curated_through_the_facade() {
    let root_artifact = worth_foundational::declared_foundational_boundary();
    let facade_artifact = worth_foundational::facade::declared_foundational_boundary();

    assert_eq!(root_artifact.payload(), facade_artifact.payload());
}
