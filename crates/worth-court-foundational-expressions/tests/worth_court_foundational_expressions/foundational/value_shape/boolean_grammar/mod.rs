// Actual array aliases, platform indices and named enum payloads share one wire grammar.
#[allow(unused_imports)]
use crate::value_shape::ordinary_context::boolean_grammar::*;
pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering};
pub(crate) use {
    worth_foundational::facade::decode_canonical_value,
    worth_foundational::facade::decode_canonical_value_with_admission,
    worth_foundational::facade::encode_canonical_value, worth_foundational::facade::CanonicalValue,
    worth_foundational::facade::ValueCodecDenial as D,
    worth_foundational::facade::ValueDecodeBounds, worth_foundational::facade::WorthValue,
};

#[test]
fn named_variant_and_nested_arrays_change_actual_wire_and_admitted_value() {
    let value = crate::value_shape::ordinary_context::boolean_grammar::Meaning::Face {
        entity: [9, 17, 4],
        nodes: [[1, 2, 3], [4, 5, 6]],
        contributors: vec![(7, [11, 12, 13, 14, 15, 16])],
    };
    assert_eq!(encode_canonical_value(&value, BOUNDS).unwrap(), RAW);
    let mut calls = 0;
    let decoded = decode_canonical_value_with_admission::<Meaning>(RAW, BOUNDS, &mut |_, _| {
        calls += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(decoded, value);
    assert!(
        calls > 6,
        "the nested array/payload seeds use the original loan"
    );
    let shape = format!(
        "{:?}",
        crate::value_shape::ordinary_context::boolean_grammar::Meaning::value_shape()
    );
    assert!(shape.contains("source"));
    assert!(shape.contains("Tuple"));
    for changed in [
        RAW.replace("[9,17,4]", "[9,17]"),
        RAW.replace("[9,17,4]", "[9,17,4,8]"),
        RAW.replace("\"source\"", "\"unknown\""),
        RAW.replace("\"Face\"", "\"Missing\""),
    ] {
        assert!(decode_canonical_value::<Meaning>(&changed, BOUNDS).is_err());
    }
    assert_eq!(
        decode_canonical_value_with_admission::<Meaning>(RAW, BOUNDS, &mut |_, _| Err(
            worth_foundational::facade::ValueCodecDenial::Admission
        )),
        Err(worth_foundational::facade::ValueCodecDenial::Admission)
    );
}

#[test]
fn platform_index_is_checked_against_the_u64_wire() {
    let raw = u64::MAX.to_string();
    match usize::try_from(u64::MAX) {
        Ok(value) => {
            assert_eq!(
                decode_canonical_value::<usize>(&raw, BOUNDS).unwrap(),
                value
            );
            assert_eq!(encode_canonical_value(&value, BOUNDS).unwrap(), raw);
        }
        Err(_) => assert_eq!(
            decode_canonical_value::<usize>(&raw, BOUNDS),
            Err(worth_foundational::facade::ValueCodecDenial::Malformed)
        ),
    }
    for raw in ["-1", "18446744073709551616", "1.5"] {
        assert_eq!(
            decode_canonical_value::<usize>(raw, BOUNDS),
            Err(worth_foundational::facade::ValueCodecDenial::Malformed)
        );
    }
    assert_eq!(usize::value_shape(), u64::value_shape());
}
#[test]
fn partial_fixed_array_refusal_drops_only_its_original_decoded_elements() {
    for (raw, expected) in [("[1,2]", 2), ("[1,2,3,4]", 3)] {
        DROPPED.store(0, std::sync::atomic::Ordering::SeqCst);
        assert!(decode_canonical_value::<[Element; 3]>(raw, BOUNDS).is_err());
        assert_eq!(DROPPED.load(std::sync::atomic::Ordering::SeqCst), expected);
    }
}
#[test]
fn explicit_domain_field_delegate_preserves_wire_constructor_and_original_refusal() {
    let value = Delegated {
        chosen: crate::value_shape::ordinary_context::boolean_grammar::DomainValue::try_new(17)
            .unwrap(),
    };
    let raw = r#"{"chosen":17}"#;
    assert_eq!(encode_canonical_value(&value, BOUNDS).unwrap(), raw);
    assert_eq!(serde_json::to_string(&value.chosen).unwrap(), "17");
    assert_eq!(
        decode_canonical_value::<Delegated>(raw, BOUNDS).unwrap(),
        value
    );
    assert_eq!(
        decode_canonical_value_with_admission::<Delegated>(raw, BOUNDS, &mut |_, _| Ok(()))
            .unwrap(),
        value
    );
    assert_eq!(
        decode_canonical_value::<Delegated>(r#"{"chosen":0}"#, BOUNDS),
        Err(worth_foundational::facade::ValueCodecDenial::NonCanonical)
    );
    assert_eq!(
        encode_canonical_value(
            &Delegated {
                chosen: DomainValue(0)
            },
            BOUNDS
        ),
        Err(worth_foundational::facade::ValueCodecDenial::NonCanonical)
    );
    assert_eq!(
        decode_canonical_value_with_admission::<Delegated>(raw, BOUNDS, &mut |_, _| Err(
            worth_foundational::facade::ValueCodecDenial::Admission
        )),
        Err(worth_foundational::facade::ValueCodecDenial::Admission)
    );
}
