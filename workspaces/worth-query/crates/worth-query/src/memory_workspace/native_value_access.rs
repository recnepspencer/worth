use worth_foundational::facade::{AspectKey, AspectValue, CanonicalFieldPath, FieldKey};

use super::WorthQueryEntity;

pub(crate) fn aspect_relative_scalar<'a>(
    entity: &'a WorthQueryEntity,
    aspect: &AspectKey,
    field_path: &CanonicalFieldPath,
) -> Option<&'a AspectValue> {
    if let [field] = field_path.fields()
        && let Some(value) = entity
            .struct_aspect_value(aspect)
            .and_then(|value| value.get(field))
    {
        return Some(value);
    }
    normalized_native_storage_path(aspect, field_path)
        .as_ref()
        .and_then(|path| entity.scalar_value_at(path))
}

pub(crate) fn normalized_native_storage_path(
    aspect: &AspectKey,
    field_path: &CanonicalFieldPath,
) -> Option<CanonicalFieldPath> {
    let mut fields = aspect
        .as_str()
        .split('.')
        .map(|segment| FieldKey::new(segment.to_owned()))
        .collect::<Option<Vec<_>>>()?;
    fields.extend(field_path.fields().iter().cloned());
    CanonicalFieldPath::new(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_workspace::admit_authored_entity_label;
    use std::collections::BTreeMap;

    #[test]
    fn dotted_aspect_reads_the_exact_canonical_scalar_path() {
        let aspect = AspectKey::new("Portfolio.Facts").unwrap();
        let relative = CanonicalFieldPath::new([FieldKey::new("desk").unwrap()]).unwrap();
        let storage = CanonicalFieldPath::new([
            FieldKey::new("Portfolio").unwrap(),
            FieldKey::new("Facts").unwrap(),
            FieldKey::new("desk").unwrap(),
        ])
        .unwrap();
        let entity = WorthQueryEntity::from_native_field_values(
            admit_authored_entity_label("position-1"),
            BTreeMap::from([(storage, AspectValue::String("rates".into()))]),
        );
        let expected = AspectValue::String("rates".into());

        assert_eq!(
            aspect_relative_scalar(&entity, &aspect, &relative),
            Some(&expected)
        );
    }
}
