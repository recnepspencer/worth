//! Persisted struct-field grammar, independently encoded by its former owner shape.
use serde::Serialize;
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectValue, FieldKey, StructAspectValue};

#[derive(Serialize)]
struct LegacyWire {
    fields: BTreeMap<FieldKey, AspectValue>,
}

#[test]
fn named_and_compact_struct_maps_preserve_legacy_bytes_and_readmission() {
    let fields = [
        (FieldKey::new("zulu").unwrap(), AspectValue::Int64(-42)),
        (FieldKey::new("active").unwrap(), AspectValue::Bool(true)),
        (
            FieldKey::new("name").unwrap(),
            AspectValue::String("réel 🦀".into()),
        ),
        (FieldKey::new("count").unwrap(), AspectValue::UInt64(500)),
    ];
    let legacy = LegacyWire {
        fields: fields.clone().into_iter().collect(),
    };
    let value = StructAspectValue::new(fields).unwrap();
    let encodings = [
        (
            "named",
            rmp_serde::to_vec_named(&legacy).unwrap(),
            rmp_serde::to_vec_named(&value).unwrap(),
        ),
        (
            "compact",
            rmp_serde::to_vec(&legacy).unwrap(),
            rmp_serde::to_vec(&value).unwrap(),
        ),
    ];
    for (encoding, old_bytes, new_bytes) in encodings {
        assert_eq!(
            new_bytes, old_bytes,
            "{encoding} persisted map grammar changed"
        );
        for bytes in [&old_bytes, &new_bytes] {
            let decoded: StructAspectValue = rmp_serde::from_slice(bytes).unwrap();
            assert_eq!(decoded, value, "{encoding} changed value meaning");
            assert_eq!(
                decoded
                    .fields()
                    .map(|(key, _)| key.as_str())
                    .collect::<Vec<_>>(),
                ["active", "count", "name", "zulu"]
            );
            for (key, expected) in &legacy.fields {
                assert_eq!(decoded.get(key), Some(expected), "{encoding} field {key:?}");
            }
            assert_eq!(decoded.get(&FieldKey::new("missing").unwrap()), None);
        }
    }
}
