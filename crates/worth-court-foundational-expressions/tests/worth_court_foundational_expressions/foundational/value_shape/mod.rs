



#[allow(unused_imports)] use crate::value_shape::ordinary_context::*;
#[allow(unused_imports)]
pub(crate) use {};



pub(crate) use {worth_foundational::facade::decode_canonical_value, worth_foundational::facade::encode_canonical_value, worth_foundational::facade::CanonicalValue, worth_foundational::facade::ValueCodecDenial, worth_foundational::facade::ValueDecodeBounds, worth_foundational::facade::WorthValue};

#[test]
fn nested_nondefault_value_has_one_exact_wire_and_seed() {
    assert_eq!(encode_canonical_value(&packet(), BOUNDS).unwrap(), RAW);
    assert_eq!(
        decode_canonical_value::<Packet>(RAW, BOUNDS).unwrap(),
        packet()
    );
    assert_eq!(
        worth_foundational::facade::ValueShapeDefinition::of::<Packet>()
            .inspect()
            .unwrap(),
        worth_foundational::facade::ValueShapeDepth::Finite(4)
    );
}

#[test]
fn enum_variant_form_is_part_of_shape_and_preserves_nullable_payloads() {
    assert_eq!(
        encode_canonical_value(&crate::value_shape::ordinary_context::UnitVariant::Choice, BOUNDS).unwrap(),
        r#""Choice""#
    );
    assert_eq!(
        encode_canonical_value(&crate::value_shape::ordinary_context::UnitPayload::Choice(()), BOUNDS).unwrap(),
        r#"{"Choice":null}"#
    );
    assert_eq!(
        decode_canonical_value::<UnitPayload>(r#"{"Choice":null}"#, BOUNDS).unwrap(),
        crate::value_shape::ordinary_context::UnitPayload::Choice(())
    );
    assert_ne!(crate::value_shape::ordinary_context::UnitVariant::value_shape(), crate::value_shape::ordinary_context::UnitPayload::value_shape());
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
            Err(worth_foundational::facade::ValueCodecDenial::Malformed)
        );
    }
    assert_eq!(
        decode_canonical_value::<Packet>(&format!(" {RAW}"), BOUNDS),
        Err(worth_foundational::facade::ValueCodecDenial::NonCanonical)
    );
}

#[test]
fn finite_representation_limits_refuse_before_owned_growth() {
    let mut too_large = packet();
    too_large.children = vec![too_large.children[0].clone(); 3];
    assert_eq!(
        encode_canonical_value(&too_large, BOUNDS),
        Err(worth_foundational::facade::ValueCodecDenial::SequenceLength)
    );
    let raw = RAW.replace("nested", &"x".repeat(33));
    assert!(decode_canonical_value::<Packet>(&raw, BOUNDS).is_err());
    assert_eq!(
        decode_canonical_value::<Packet>(RAW, worth_foundational::facade::ValueDecodeBounds::new(16, 32, 2, 12)),
        Err(worth_foundational::facade::ValueCodecDenial::WireBytes)
    );
    assert_eq!(
        encode_canonical_value(&packet(), worth_foundational::facade::ValueDecodeBounds::new(1024, 32, 2, 1)),
        Err(worth_foundational::facade::ValueCodecDenial::Depth)
    );
    // An additional sequence member is denied before its malformed payload is decoded.
    let mut decoder = serde_json::Deserializer::from_str("[1,2,{\"not-a-number\":0}]");
    let error =
        <Vec<u64> as CanonicalValue>::deserialize_bounded(&mut decoder, BOUNDS).unwrap_err();
    assert!(error.to_string().contains("sequence length bound"));
}
#[path = "admitted/mod.rs"]
pub(crate) mod admitted;
#[path = "boolean_grammar/mod.rs"]
pub(crate) mod boolean_grammar;
#[path = "recursive/mod.rs"]
pub(crate) mod recursive;
