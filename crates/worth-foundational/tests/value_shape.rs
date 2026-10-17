//! Canonical values retain nondefault meaning under finite representation bounds.
#[path = "value_shape/compiler.rs"]
mod compiler;

use worth_foundational::facade::{
    decode_canonical_value, encode_canonical_value, CanonicalValue, ValueCodecDenial,
    ValueDecodeBounds, WorthValue,
};

#[derive(Clone, Debug, PartialEq, WorthValue)]
struct Packet {
    #[serde(rename = "selection")]
    value: Choice,
    children: Vec<Child>,
}
#[derive(Clone, Debug, PartialEq, WorthValue)]
enum Choice {
    Empty,
    #[serde(rename = "chosen")]
    Selected(Child),
}
#[derive(Clone, Debug, PartialEq, WorthValue)]
struct Child {
    count: i32,
    text: String,
    present: Option<bool>,
}

const BOUNDS: ValueDecodeBounds = ValueDecodeBounds::new(1024, 32, 2, 12);
const RAW: &str = r#"{"selection":{"chosen":{"count":-17,"text":"nested","present":false}},"children":[{"count":9,"text":"δ","present":null}]}"#;

fn packet() -> Packet {
    Packet {
        value: Choice::Selected(Child {
            count: -17,
            text: "nested".into(),
            present: Some(false),
        }),
        children: vec![Child {
            count: 9,
            text: "δ".into(),
            present: None,
        }],
    }
}

#[test]
fn nested_nondefault_value_has_one_exact_wire_and_seed() {
    assert_eq!(encode_canonical_value(&packet(), BOUNDS).unwrap(), RAW);
    assert_eq!(
        decode_canonical_value::<Packet>(RAW, BOUNDS).unwrap(),
        packet()
    );
    assert_eq!(Packet::SHAPE_DEPTH, 4);
}

#[derive(Debug, PartialEq, WorthValue)]
enum UnitVariant {
    Choice,
}
#[derive(Debug, PartialEq, WorthValue)]
enum UnitPayload {
    Choice(()),
}

#[test]
fn enum_variant_form_is_part_of_shape_and_preserves_nullable_payloads() {
    assert_eq!(
        encode_canonical_value(&UnitVariant::Choice, BOUNDS).unwrap(),
        r#""Choice""#
    );
    assert_eq!(
        encode_canonical_value(&UnitPayload::Choice(()), BOUNDS).unwrap(),
        r#"{"Choice":null}"#
    );
    assert_eq!(
        decode_canonical_value::<UnitPayload>(r#"{"Choice":null}"#, BOUNDS).unwrap(),
        UnitPayload::Choice(())
    );
    assert_ne!(UnitVariant::value_shape(), UnitPayload::value_shape());
}

#[test]
fn seed_refuses_missing_duplicate_unknown_and_noncanonical_input() {
    for raw in [
        RAW.replace("\"count\":-17,", ""),
        RAW.replace("\"count\":-17,", "\"count\":-17,\"count\":-18,"),
        RAW.replace("\"count\":-17,", "\"extra\":0,\"count\":-17,"),
        RAW.replace("chosen", "unknown"),
    ] {
        assert_eq!(
            decode_canonical_value::<Packet>(&raw, BOUNDS),
            Err(ValueCodecDenial::Malformed)
        );
    }
    assert_eq!(
        decode_canonical_value::<Packet>(&format!(" {RAW}"), BOUNDS),
        Err(ValueCodecDenial::NonCanonical)
    );
}

#[test]
fn finite_representation_limits_refuse_before_owned_growth() {
    let mut too_large = packet();
    too_large.children = vec![too_large.children[0].clone(); 3];
    assert_eq!(
        encode_canonical_value(&too_large, BOUNDS),
        Err(ValueCodecDenial::SequenceLength)
    );
    let raw = RAW.replace("nested", &"x".repeat(33));
    assert!(decode_canonical_value::<Packet>(&raw, BOUNDS).is_err());
    assert_eq!(
        decode_canonical_value::<Packet>(RAW, ValueDecodeBounds::new(16, 32, 2, 12)),
        Err(ValueCodecDenial::WireBytes)
    );
    assert_eq!(
        encode_canonical_value(&packet(), ValueDecodeBounds::new(1024, 32, 2, 1)),
        Err(ValueCodecDenial::Depth)
    );
    // An additional sequence member is denied before its malformed payload is decoded.
    let mut decoder = serde_json::Deserializer::from_str("[1,2,{\"not-a-number\":0}]");
    let error =
        <Vec<u64> as CanonicalValue>::deserialize_bounded(&mut decoder, BOUNDS).unwrap_err();
    assert!(error.to_string().contains("sequence length bound"));
}
