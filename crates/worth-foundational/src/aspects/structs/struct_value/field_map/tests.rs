use super::*;

#[test]
fn duplicate_map_entries_move_the_first_key_and_last_value() {
    let mut first_key = String::with_capacity(128);
    first_key.push_str("field");
    let first_key = FieldKey::new(first_key).unwrap();
    let original_key = first_key.as_str().as_ptr();
    let rows = vec![
        DecodedField {
            key: first_key,
            value: AspectValue::UInt64(1),
            ordinal: 0,
        },
        DecodedField {
            key: FieldKey::new("field").unwrap(),
            value: AspectValue::UInt64(7),
            ordinal: 1,
        },
    ];
    let fields = canonical_fields::<serde::de::value::Error>(rows).unwrap();
    assert_eq!(fields[0].0.as_str().as_ptr(), original_key);
    assert_eq!(fields[0].1, AspectValue::UInt64(7));
}
