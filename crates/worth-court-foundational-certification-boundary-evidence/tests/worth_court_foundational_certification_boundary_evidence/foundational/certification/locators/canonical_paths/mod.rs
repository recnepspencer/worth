use {worth_foundational::AspectFieldLocator, worth_foundational::AspectKey, worth_foundational::AspectLocator, worth_foundational::AspectMask, worth_foundational::AspectMaskLocator, worth_foundational::AspectValueLocator, worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId, worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryMismatchLocator, worth_foundational::BoundarySourceLocator, worth_foundational::CanonicalFieldPath, worth_foundational::DiagnosticMask, worth_foundational::FieldKey, worth_foundational::LocatorAuthority, worth_foundational::MutationMask};

use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key};

#[test]
fn equivalent_mask_locators_canonicalize_path_order() {
    let left_mask = AspectMask::<MutationMask>::new([
        worth_foundational::CanonicalFieldPath::single(field("title")),
        worth_foundational::CanonicalFieldPath::single(field("done")),
    ]);
    let right_mask = AspectMask::<MutationMask>::new([
        worth_foundational::CanonicalFieldPath::single(field("done")),
        worth_foundational::CanonicalFieldPath::single(field("title")),
        worth_foundational::CanonicalFieldPath::single(field("done")),
    ]);

    let left = worth_foundational::AspectMaskLocator::mutation(
        worth_foundational::LocatorAuthority::Authoritative,
        key("task.summary"),
        &left_mask,
    );
    let right = worth_foundational::AspectMaskLocator::mutation(
        worth_foundational::LocatorAuthority::Authoritative,
        key("task.summary"),
        &right_mask,
    );

    assert_eq!(left, right);
    assert_eq!(
        left.paths(),
        &[
            worth_foundational::CanonicalFieldPath::single(field("done")),
            worth_foundational::CanonicalFieldPath::single(field("title")),
        ]
    );
}

#[test]
fn field_value_and_source_locators_share_the_same_structural_locus() {
    let field_path =
        worth_foundational::CanonicalFieldPath::new([field("parent"), field("child")]).expect("non-empty field path");
    let field_locator = worth_foundational::AspectFieldLocator::new(
        worth_foundational::LocatorAuthority::Authoritative,
        key("task.summary"),
        field_path.clone(),
    );
    let value_locator = worth_foundational::AspectValueLocator::struct_field(field_locator.clone());
    let source_locator = worth_foundational::BoundarySourceLocator::aspect_field(field_locator.clone());
    let mismatch_locator = worth_foundational::BoundaryMismatchLocator::aspect_field(field_locator.clone());

    assert!(matches!(
        value_locator,
        worth_foundational::AspectValueLocator::StructField(ref located)
            if located.field_path() == &field_path
                && located.aspect().aspect_key() == &key("task.summary")
    ));
    assert_eq!(
        source_locator,
        worth_foundational::BoundarySourceLocator::aspect_field(field_locator.clone())
    );
    assert_eq!(
        mismatch_locator,
        worth_foundational::BoundaryMismatchLocator::aspect_field(field_locator)
    );
}

#[test]
fn artifact_locators_preserve_artifact_field_category() {
    let artifact_locator =
        worth_foundational::BoundaryArtifactLocator::new(worth_foundational::BoundaryArtifactId::new(42), worth_foundational::BoundaryArtifactField::Payload);
    let source_locator = worth_foundational::BoundarySourceLocator::boundary_artifact(artifact_locator);

    assert_eq!(artifact_locator.artifact_id(), worth_foundational::BoundaryArtifactId::new(42));
    assert_eq!(artifact_locator.field(), worth_foundational::BoundaryArtifactField::Payload);
    assert_eq!(
        source_locator,
        worth_foundational::BoundarySourceLocator::boundary_artifact(artifact_locator)
    );
}

#[test]
fn locator_authority_is_part_of_locator_identity() {
    let authoritative = worth_foundational::AspectLocator::new(worth_foundational::LocatorAuthority::Authoritative, key("count"));
    let projected = worth_foundational::AspectLocator::new(worth_foundational::LocatorAuthority::Projected, key("count"));

    assert_ne!(authoritative, projected);
}

#[test]
fn diagnostic_mask_locators_remain_mode_specific() {
    let mask = AspectMask::<DiagnosticMask>::new([worth_foundational::CanonicalFieldPath::single(field("note"))]);
    let located = worth_foundational::AspectMaskLocator::diagnostic(
        worth_foundational::LocatorAuthority::SupportOnly,
        worth_foundational::AspectKey::new("task.summary").expect("valid key"),
        &mask,
    );

    assert_eq!(located.authority(), worth_foundational::LocatorAuthority::SupportOnly);
    assert_eq!(located.aspect_key(), &key("task.summary"));
    assert_eq!(
        located.paths(),
        &[worth_foundational::CanonicalFieldPath::single(worth_foundational::FieldKey::new("note").unwrap())]
    );
}
