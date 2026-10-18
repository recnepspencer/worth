use worth_foundational::{
    AspectFieldLocator, AspectKey, AspectLocator, AspectMask, AspectMaskLocator,
    AspectValueLocator, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    BoundaryMismatchLocator, BoundarySourceLocator, CanonicalFieldPath, DiagnosticMask, FieldKey,
    LocatorAuthority, MutationMask,
};

use crate::certification::foundational_vocabulary::{field, key};

#[test]
fn equivalent_mask_locators_canonicalize_path_order() {
    let left_mask = AspectMask::<MutationMask>::new([
        CanonicalFieldPath::single(field("title")),
        CanonicalFieldPath::single(field("done")),
    ]);
    let right_mask = AspectMask::<MutationMask>::new([
        CanonicalFieldPath::single(field("done")),
        CanonicalFieldPath::single(field("title")),
        CanonicalFieldPath::single(field("done")),
    ]);

    let left = AspectMaskLocator::mutation(
        LocatorAuthority::Authoritative,
        key("task.summary"),
        &left_mask,
    );
    let right = AspectMaskLocator::mutation(
        LocatorAuthority::Authoritative,
        key("task.summary"),
        &right_mask,
    );

    assert_eq!(left, right);
    assert_eq!(
        left.paths(),
        &[
            CanonicalFieldPath::single(field("done")),
            CanonicalFieldPath::single(field("title")),
        ]
    );
}

#[test]
fn field_value_and_source_locators_share_the_same_structural_locus() {
    let field_path =
        CanonicalFieldPath::new([field("parent"), field("child")]).expect("non-empty field path");
    let field_locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        key("task.summary"),
        field_path.clone(),
    );
    let value_locator = AspectValueLocator::struct_field(field_locator.clone());
    let source_locator = BoundarySourceLocator::aspect_field(field_locator.clone());
    let mismatch_locator = BoundaryMismatchLocator::aspect_field(field_locator.clone());

    assert!(matches!(
        value_locator,
        AspectValueLocator::StructField(ref located)
            if located.field_path() == &field_path
                && located.aspect().aspect_key() == &key("task.summary")
    ));
    assert_eq!(
        source_locator,
        BoundarySourceLocator::aspect_field(field_locator.clone())
    );
    assert_eq!(
        mismatch_locator,
        BoundaryMismatchLocator::aspect_field(field_locator)
    );
}

#[test]
fn artifact_locators_preserve_artifact_field_category() {
    let artifact_locator =
        BoundaryArtifactLocator::new(BoundaryArtifactId::new(42), BoundaryArtifactField::Payload);
    let source_locator = BoundarySourceLocator::boundary_artifact(artifact_locator);

    assert_eq!(artifact_locator.artifact_id(), BoundaryArtifactId::new(42));
    assert_eq!(artifact_locator.field(), BoundaryArtifactField::Payload);
    assert_eq!(
        source_locator,
        BoundarySourceLocator::boundary_artifact(artifact_locator)
    );
}

#[test]
fn locator_authority_is_part_of_locator_identity() {
    let authoritative = AspectLocator::new(LocatorAuthority::Authoritative, key("count"));
    let projected = AspectLocator::new(LocatorAuthority::Projected, key("count"));

    assert_ne!(authoritative, projected);
}

#[test]
fn diagnostic_mask_locators_remain_mode_specific() {
    let mask = AspectMask::<DiagnosticMask>::new([CanonicalFieldPath::single(field("note"))]);
    let located = AspectMaskLocator::diagnostic(
        LocatorAuthority::SupportOnly,
        AspectKey::new("task.summary").expect("valid key"),
        &mask,
    );

    assert_eq!(located.authority(), LocatorAuthority::SupportOnly);
    assert_eq!(located.aspect_key(), &key("task.summary"));
    assert_eq!(
        located.paths(),
        &[CanonicalFieldPath::single(FieldKey::new("note").unwrap())]
    );
}
