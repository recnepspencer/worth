pub use std::collections::BTreeMap;

pub use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey, InternedString};
pub use worth_query::facade::domain::{WorthQueryGraphReadMaterial, WorthQueryGraphReadRow};

pub fn graph_read_material(label: &str) -> WorthQueryGraphReadMaterial {
    let identity_path = CanonicalFieldPath::single(FieldKey::new("id").unwrap());
    let fields = BTreeMap::from([(
        identity_path,
        AspectValue::String(InternedString::from(label)),
    )]);
    let row = WorthQueryGraphReadRow::from_native_fields(label, fields).unwrap();
    WorthQueryGraphReadMaterial::new([row])
}
