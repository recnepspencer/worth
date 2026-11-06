use super::FieldKey;
use crate::aspects::structs::CanonicalFieldPath;

#[test]
fn flattened_leaf_names_cannot_alias_a_multi_component_path() {
    let flat = CanonicalFieldPath::single(FieldKey::new("samples[1].value").unwrap());
    let nested = CanonicalFieldPath::new([
        FieldKey::new("samples[1]").unwrap(),
        FieldKey::new("value").unwrap(),
    ])
    .unwrap();
    assert_ne!(flat, nested);
    assert_eq!(flat.fields().len(), 1);
    assert_eq!(flat.fields()[0].as_str(), "samples[1].value");
    assert!(FieldKey::new(" ").is_none());
    assert!(FieldKey::new("has whitespace").is_none());
}
