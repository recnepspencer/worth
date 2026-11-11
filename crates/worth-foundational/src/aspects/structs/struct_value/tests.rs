use super::*;
use crate::InternedString;

fn field(name: &str, value: u64) -> (FieldKey, AspectValue) {
    (FieldKey::new(name).unwrap(), AspectValue::UInt64(value))
}

#[test]
fn ordinary_construction_stops_at_the_first_encountered_duplicate() {
    let entries = [field("z", 1), field("a", 2), field("z", 3), field("a", 4)];
    let mut consumed = 0;
    let input = entries.into_iter().inspect(|_| {
        consumed += 1;
        assert!(
            consumed <= 3,
            "the invalid iterator suffix must remain untouched"
        );
    });
    assert_eq!(
        StructAspectValue::new(input),
        Err(StructAspectValueConstructionDenial::DuplicateField(
            FieldKey::new("z").unwrap()
        ))
    );
    assert_eq!(consumed, 3);
}

#[test]
fn sorted_construction_returns_the_same_rejected_allocation() {
    for fields in [
        vec![field("a", 1), field("a", 2)],
        vec![field("a", 1), field("z", 2), field("b", 3)],
    ] {
        let fields = fields.into_boxed_slice();
        let original = fields.as_ptr();
        let invalid = if fields.len() == 2 { 0 } else { 1 };
        let denied = StructAspectValue::from_sorted_fields(fields).unwrap_err();
        assert_eq!(denied.fields().as_ptr(), original);
        assert_eq!(denied.invalid_pair_index(), invalid);
        let returned = denied.into_fields();
        assert_eq!(returned.as_ptr(), original);
    }
}

#[test]
fn canonical_fields_keep_lookup_clone_and_retained_capacity_meaning() {
    let mut key = String::with_capacity(96);
    key.push('a');
    let key = FieldKey::new(key).unwrap();
    let mut text = String::with_capacity(192);
    text.push_str("value");
    let nested = key.owned_allocation_capacity_bytes() + text.capacity();
    let fields = vec![
        (key, AspectValue::String(InternedString::Raw(text))),
        field("z", 9),
    ]
    .into_boxed_slice();
    let nested = nested + fields[1].0.owned_allocation_capacity_bytes();
    let value = StructAspectValue::from_sorted_fields(fields).unwrap();
    assert_eq!(
        value.owned_allocation_capacity_bytes(),
        StructAspectValue::sorted_construction_storage_bytes(2).unwrap() + nested
    );
    assert_eq!(
        value.get(&FieldKey::new("z").unwrap()),
        Some(&AspectValue::UInt64(9))
    );
    assert_eq!(value.get(&FieldKey::new("b").unwrap()), None);
    let cloned = value.clone();
    assert_eq!(cloned, value);
    // c65d042399 / 6df49ed086 retire deep-copy backing: the immutable
    // value is shared, and the successor still owns it after its predecessor.
    assert!(std::ptr::eq(
        cloned.get(&FieldKey::new("a").unwrap()).unwrap(),
        value.get(&FieldKey::new("a").unwrap()).unwrap()
    ));
    drop(value);
    assert_eq!(
        cloned.get(&FieldKey::new("z").unwrap()),
        Some(&AspectValue::UInt64(9))
    );
    assert_eq!(
        cloned
            .fields()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        ["a", "z"]
    );
    assert!(StructAspectValue::from_sorted_fields(Box::new([]))
        .unwrap()
        .is_empty());
}

#[test]
fn serde_keeps_map_wire_order_and_legacy_duplicate_values() {
    let input =
        r#"{"ignored":[1,2],"fields":{"z":{"UInt64":1},"a":{"Bool":true},"z":{"UInt64":9}}}"#;
    let value: StructAspectValue = serde_json::from_str(input).unwrap();
    assert_eq!(value.len(), 2);
    assert_eq!(
        value.get(&FieldKey::new("z").unwrap()),
        Some(&AspectValue::UInt64(9))
    );
    assert_eq!(
        serde_json::to_string(&value).unwrap(),
        r#"{"fields":{"a":{"Bool":true},"z":{"UInt64":9}}}"#
    );
    for invalid in [r#"{}"#, r#"{"fields":{},"fields":{}}"#, r#"{"fields":[]}"#] {
        assert!(
            serde_json::from_str::<StructAspectValue>(invalid).is_err(),
            "{invalid}"
        );
    }
    let compact: StructAspectValue = serde_json::from_str(r#"[{"a":{"UInt64":2}}]"#).unwrap();
    assert_eq!(
        compact.get(&FieldKey::new("a").unwrap()),
        Some(&AspectValue::UInt64(2))
    );
}

#[test]
fn decode_bounds_cover_backing_growth_and_reject_layout_overflow() {
    assert_eq!(
        StructAspectValue::decode_storage_bounds(0),
        Some(StructAspectValueStorageBounds {
            retained_fields_bytes: 0,
            transient_decode_bytes: 0,
        })
    );
    let mut prior = 0;
    for count in [1usize, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33] {
        let bounds = StructAspectValue::decode_storage_bounds(count).unwrap();
        let peak = bounds.retained_fields_bytes + bounds.transient_decode_bytes;
        assert!(peak >= prior);
        assert!(peak >= count * size_of::<(FieldKey, AspectValue)>());
        let wire = format!(
            "{{\"fields\":{{{}}}}}",
            (0..count)
                .map(|i| format!("\"f{i:03}\":{{\"UInt64\":{i}}}"))
                .collect::<Vec<_>>()
                .join(",")
        );
        let decoded: StructAspectValue = serde_json::from_str(&wire).unwrap();
        assert_eq!(
            StructAspectValue::sorted_construction_storage_bytes(decoded.len()).unwrap(),
            bounds.retained_fields_bytes
        );
        prior = peak;
    }
    assert!(StructAspectValue::decode_storage_bounds(usize::MAX).is_none());
    assert!(StructAspectValue::decode_storage_bounds(isize::MAX as usize).is_none());
}

#[test]
fn shape_construction_preserves_sorted_declarations_and_rejects_duplicate_keys() {
    use crate::aspects::{AbsenceLaw, AspectEvolutionPolicy, FieldRequirement};
    use crate::values::ScalarAspectType;
    let declaration = |name: &str, value_type| {
        FieldDeclaration::new(
            FieldKey::new(name).unwrap(),
            value_type,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::Frozen,
        )
        .unwrap()
    };
    let a = declaration("a", ScalarAspectType::String);
    let m = declaration("m", ScalarAspectType::Bool);
    let z = declaration("z", ScalarAspectType::UInt64);
    let shape = StructAspectShape::new([z.clone(), a.clone(), m.clone()]).unwrap();
    assert_eq!(shape.fields(), &[a.clone(), m.clone(), z.clone()]);
    assert_eq!(shape.field(a.key()), Some(&a));
    // Unequal declarations still duplicate the same field key after sorting.
    let repeated_a = declaration("a", ScalarAspectType::Int64);
    assert!(StructAspectShape::new([z, a, m, repeated_a]).is_none());
    assert!(StructAspectShape::new([]).unwrap().fields().is_empty());
}
