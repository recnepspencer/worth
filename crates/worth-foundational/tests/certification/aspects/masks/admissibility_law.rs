use worth_foundational::{
    AbsenceLaw, AspectContract, AspectMask, CanonicalFieldPath, FieldDeclaration, FieldRequirement,
    MaskAdmissibilityDenial, MutationMask, ProjectionMask, ScalarAspectType, StructAspectShape,
};

use crate::certification::foundational_vocabulary::{field, identity, key, revision};

#[test]
fn masks_are_mode_typed_and_shape_admitted() {
    let title = field("title");
    let title_field = FieldDeclaration::new(
        title.clone(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("coherent field law");
    let shape = StructAspectShape::new([title_field]).expect("unique fields");
    let struct_contract =
        AspectContract::struct_aspect(key("task.summary"), identity(2), revision(1), shape);
    let field_mask = AspectMask::<ProjectionMask>::new([CanonicalFieldPath::single(title.clone())]);

    assert_eq!(struct_contract.admits_projection_mask(&field_mask), Ok(()));

    let scalar_contract = AspectContract::scalar(
        key("task.title"),
        identity(3),
        revision(1),
        ScalarAspectType::String,
    );
    assert_eq!(
        scalar_contract.admits_projection_mask(&field_mask),
        Err(MaskAdmissibilityDenial::FieldMaskRequiresStruct)
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
    let shape = StructAspectShape::new([FieldDeclaration::new(
        title,
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("coherent field law")])
    .expect("unique fields");
    let struct_contract =
        AspectContract::struct_aspect(key("task.summary"), identity(2), revision(1), shape);
    let unknown_field_mask =
        AspectMask::<ProjectionMask>::new([CanonicalFieldPath::single(field("surprise"))]);

    assert_eq!(
        struct_contract.admits_projection_mask(&unknown_field_mask),
        Err(MaskAdmissibilityDenial::UnknownField)
    );
}

#[test]
fn mask_paths_canonicalize_independent_of_input_order_and_duplicates() {
    let a = field("a");
    let b = field("b");
    let mask = AspectMask::<ProjectionMask>::new([
        CanonicalFieldPath::single(b.clone()),
        CanonicalFieldPath::single(a.clone()),
        CanonicalFieldPath::single(a),
    ]);

    let materialized: Vec<_> = mask
        .paths()
        .iter()
        .map(|path| path.fields()[0].as_str())
        .collect();

    assert_eq!(materialized, vec!["a", "b"]);
}
