use worth_foundational::facade::{
    decode_canonical_value, encode_canonical_value, CanonicalValue, ValueDecodeBounds, WorthValue,
};

#[derive(Debug, PartialEq, WorthValue)]
struct Child {
    label: String,
    count: u32,
}

#[derive(Debug, PartialEq, WorthValue)]
struct Identifier(u64);

#[derive(Debug, PartialEq, WorthValue)]
struct NestedIdentifier(Identifier);

#[derive(Debug, PartialEq, WorthValue)]
struct UnitMarker;

#[derive(Debug, PartialEq, WorthValue)]
struct OneLevel {
    marker: UnitMarker,
}

#[derive(Debug, PartialEq, WorthValue)]
enum Selection {
    Empty,
    #[serde(rename = "chosen")]
    Selected(Child),
}

#[derive(Debug, PartialEq, WorthValue)]
struct SavedValue {
    id: Identifier,
    #[serde(rename = "selection")]
    selected: Selection,
    children: Vec<Child>,
    note: Option<String>,
}

fn main() {
    assert_eq!(
        worth_foundational::facade::ValueShapeDefinition::of::<Identifier>()
            .inspect()
            .unwrap(),
        worth_foundational::facade::ValueShapeDefinition::of::<u64>()
            .inspect()
            .unwrap()
    );
    assert_eq!(
        worth_foundational::facade::ValueShapeDefinition::of::<NestedIdentifier>()
            .inspect()
            .unwrap(),
        worth_foundational::facade::ValueShapeDefinition::of::<u64>()
            .inspect()
            .unwrap()
    );
    assert_eq!(NestedIdentifier::value_shape(), u64::value_shape());
    let shallow_bounds = ValueDecodeBounds::new(2048, 64, 8, 1);
    let nested_identifier = NestedIdentifier(Identifier(73));
    nested_identifier.validate_bounds(shallow_bounds).unwrap();
    assert_eq!(
        decode_canonical_value::<NestedIdentifier>("73", shallow_bounds).unwrap(),
        nested_identifier,
    );
    assert_eq!(
        worth_foundational::facade::ValueShapeDefinition::of::<UnitMarker>()
            .inspect()
            .unwrap(),
        worth_foundational::facade::ValueShapeDepth::Finite(0)
    );
    let one_level = OneLevel { marker: UnitMarker };
    one_level.validate_bounds(shallow_bounds).unwrap();
    assert_eq!(
        decode_canonical_value::<OneLevel>(r#"{"marker":null}"#, shallow_bounds).unwrap(),
        one_level,
    );

    let value = SavedValue {
        id: Identifier(73),
        selected: Selection::Selected(Child {
            label: "picked".to_owned(),
            count: 4,
        }),
        children: vec![Child {
            label: "nested".to_owned(),
            count: 9,
        }],
        note: Some("saved".to_owned()),
    };
    let bounds = ValueDecodeBounds::new(2048, 64, 8, 12);
    let wire = encode_canonical_value(&value, bounds).unwrap();
    assert_eq!(
        wire,
        r#"{"id":73,"selection":{"chosen":{"label":"picked","count":4}},"children":[{"label":"nested","count":9}],"note":"saved"}"#,
    );
    assert_eq!(
        decode_canonical_value::<SavedValue>(&wire, bounds).unwrap(),
        value
    );
    assert!(
        matches!(worth_foundational::facade::ValueShapeDefinition::of::<SavedValue>().inspect().unwrap(), worth_foundational::facade::ValueShapeDepth::Finite(depth) if depth < 12)
    );
    let _ = SavedValue::value_shape();
}
