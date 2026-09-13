use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectKey, AspectValue, CanonicalFieldPath, FieldKey, StructAspectValue,
};

use super::{
    admit_authored_entity_label, WorthQueryEntity, WorthQueryEntityNativeReplacement,
    WorthQueryEntityNativeReplacementValue,
};

#[test]
fn replacement_updates_duplicate_native_storage_without_stale_precedence() {
    let (entity, aspect, rank, desk, path) = duplicate_storage_entity();
    let replaced = entity.replace_native_values([WorthQueryEntityNativeReplacement::new(
        aspect.clone(),
        Some(rank.clone()),
        [path.clone()],
        WorthQueryEntityNativeReplacementValue::Scalar(AspectValue::UInt64(2)),
    )]);

    assert_eq!(replaced.aspect_value(&aspect), None);
    let structured = replaced.struct_aspect_value(&aspect).unwrap();
    assert_eq!(structured.get(&rank), Some(&AspectValue::UInt64(2)));
    assert_eq!(
        structured.get(&desk),
        Some(&AspectValue::String("rates".into()))
    );
    assert_eq!(
        replaced.scalar_value_at(&path),
        Some(&AspectValue::UInt64(2))
    );
}

#[test]
fn absent_replacement_clears_every_copy_and_preserves_sibling_struct_fields() {
    let (entity, aspect, rank, desk, path) = duplicate_storage_entity();
    let replaced = entity.replace_native_values([WorthQueryEntityNativeReplacement::new(
        aspect.clone(),
        Some(rank.clone()),
        [path.clone()],
        WorthQueryEntityNativeReplacementValue::Absent,
    )]);

    assert_eq!(replaced.aspect_value(&aspect), None);
    let structured = replaced.struct_aspect_value(&aspect).unwrap();
    assert_eq!(structured.get(&rank), None);
    assert_eq!(
        structured.get(&desk),
        Some(&AspectValue::String("rates".into()))
    );
    assert_eq!(replaced.scalar_value_at(&path), None);
}

fn duplicate_storage_entity() -> (
    WorthQueryEntity,
    AspectKey,
    FieldKey,
    FieldKey,
    CanonicalFieldPath,
) {
    let aspect = AspectKey::new("PortfolioFacts").unwrap();
    let rank = FieldKey::new("PortfolioRankField").unwrap();
    let desk = FieldKey::new("PortfolioDeskField").unwrap();
    let path =
        CanonicalFieldPath::new([FieldKey::new("PortfolioFacts").unwrap(), rank.clone()]).unwrap();
    let structured = StructAspectValue::new([
        (rank.clone(), AspectValue::UInt64(1)),
        (desk.clone(), AspectValue::String("rates".into())),
    ])
    .unwrap();
    let entity = WorthQueryEntity::from_aspect_projection(
        admit_authored_entity_label("duplicate-storage-row"),
        BTreeMap::from([(aspect.clone(), AspectValue::UInt64(1))]),
        BTreeMap::from([(aspect.clone(), structured)]),
        BTreeMap::from([(path.clone(), AspectValue::UInt64(1))]),
    );
    (entity, aspect, rank, desk, path)
}
