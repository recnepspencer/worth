use {
    worth_foundational::AspectMask, worth_foundational::MutationMask,
    worth_foundational::ProjectionMask,
};

use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision,
};

#[test]
fn masks_are_mode_typed_and_shape_admitted() {
    let title = field("title");
    let title_field = worth_foundational::FieldDeclaration::new(
        title.clone(),
        worth_foundational::ScalarAspectType::String,
        worth_foundational::FieldRequirement::Required,
        worth_foundational::AbsenceLaw::Required,
        worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("coherent field law");
    let shape = worth_foundational::StructAspectShape::new([title_field]).expect("unique fields");
    let struct_contract = worth_foundational::AspectContract::struct_aspect(
        key("task.summary"),
        identity(2),
        revision(1),
        shape,
    );
    let field_mask =
        AspectMask::<ProjectionMask>::new([worth_foundational::CanonicalFieldPath::single(
            title.clone(),
        )]);

    assert_eq!(struct_contract.admits_projection_mask(&field_mask), Ok(()));

    let scalar_contract = worth_foundational::AspectContract::scalar(
        key("task.title"),
        identity(3),
        revision(1),
        worth_foundational::ScalarAspectType::String,
    );
    assert_eq!(
        scalar_contract.admits_projection_mask(&field_mask),
        Err(worth_foundational::MaskAdmissibilityDenial::FieldMaskRequiresStruct)
    );

    let mutation_whole = AspectMask::<MutationMask>::whole_aspect();
    assert_eq!(
        scalar_contract.admits_mutation_mask(&mutation_whole),
        Ok(())
    );
}

#[test]
fn struct_masks_reject_unknown_fields() {
    let title = field("title");
    let shape =
        worth_foundational::StructAspectShape::new([worth_foundational::FieldDeclaration::new(
            title,
            worth_foundational::ScalarAspectType::String,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law")])
        .expect("unique fields");
    let struct_contract = worth_foundational::AspectContract::struct_aspect(
        key("task.summary"),
        identity(2),
        revision(1),
        shape,
    );
    let unknown_field_mask =
        AspectMask::<ProjectionMask>::new([worth_foundational::CanonicalFieldPath::single(field(
            "surprise",
        ))]);

    assert_eq!(
        struct_contract.admits_projection_mask(&unknown_field_mask),
        Err(worth_foundational::MaskAdmissibilityDenial::UnknownField)
    );
}

#[test]
fn mask_paths_canonicalize_independent_of_input_order_and_duplicates() {
    let a = field("a");
    let b = field("b");
    let mask = AspectMask::<ProjectionMask>::new([
        worth_foundational::CanonicalFieldPath::single(b.clone()),
        worth_foundational::CanonicalFieldPath::single(a.clone()),
        worth_foundational::CanonicalFieldPath::single(a),
    ]);

    let materialized: Vec<_> = mask
        .paths()
        .iter()
        .map(|path| path.fields()[0].as_str())
        .collect();

    assert_eq!(materialized, vec!["a", "b"]);
}
