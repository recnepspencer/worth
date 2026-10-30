// Neutral recursive tagged, named, boxed and fixed-product grammar.
#[allow(unused_imports)]
use crate::value_shape::ordinary_context::recursive::*;
pub(crate) use {
    worth_foundational::facade::decode_canonical_value,
    worth_foundational::facade::decode_canonical_value_with_admission,
    worth_foundational::facade::encode_canonical_value, worth_foundational::facade::CanonicalValue,
    worth_foundational::facade::ValueCodecDenial as D,
    worth_foundational::facade::ValueDecodeBounds, worth_foundational::facade::ValueShape,
    worth_foundational::facade::ValueShapeDefinition, worth_foundational::facade::ValueShapeDepth,
    worth_foundational::facade::WorthValue,
};

#[test]
fn ordinary_recursive_wire_passes_retired_depth_and_preserves_deep_refusal() {
    let mut value = crate::value_shape::ordinary_context::recursive::Expression::Text {
        value: "deep".into(),
    };
    let mut expected = String::new();
    for _ in 0..512 {
        value = crate::value_shape::ordinary_context::recursive::Expression::Negate {
            operand: Box::new(value),
        };
        expected.push_str(r#"{"kind":"negate","operand":"#);
    }
    expected.push_str(r#"{"kind":"text","value":"deep"}"#);
    expected.push_str(&"}".repeat(512));
    let unrestricted = worth_foundational::facade::ValueDecodeBounds::unrestricted();
    assert_eq!(
        encode_canonical_value(&value, unrestricted).unwrap(),
        expected
    );
    let decoded = decode_canonical_value_with_admission::<Expression>(
        &expected,
        unrestricted,
        &mut |_, _| Ok(()),
    )
    .unwrap();
    assert_eq!(decoded, value);
    let mut stopped = false;
    let denied = decode_canonical_value_with_admission::<Expression>(
        &expected,
        unrestricted,
        &mut |_, bytes| {
            assert!(
                !stopped,
                "original callback is not re-entered after its refusal"
            );
            if bytes == 4 {
                stopped = true;
                return Err(worth_foundational::facade::ValueCodecDenial::Admission);
            }
            Ok(())
        },
    );
    assert_eq!(
        denied,
        Err(worth_foundational::facade::ValueCodecDenial::Admission)
    );
    assert!(stopped, "refusal reached the actual deep string allocation");
    // A complete deep first sibling is dropped on a later malformed member.
    let partial = format!(
        r#"{{"kind":"group","members":[{{"label":"first","value":{expected}}},{{"label":"bad","value":{{"kind":"unknown"}}}}]}}"#
    );
    assert_eq!(
        decode_canonical_value_with_admission::<Expression>(
            &partial,
            unrestricted,
            &mut |_, _| Ok(())
        ),
        Err(worth_foundational::facade::ValueCodecDenial::Malformed)
    );
    assert_eq!(
        decode_canonical_value::<Expression>(
            &expected,
            worth_foundational::facade::ValueDecodeBounds::new(expected.len(), 64, 8, 24)
        ),
        Err(worth_foundational::facade::ValueCodecDenial::Depth)
    );
}

#[test]
fn recursive_production_seed_preserves_independent_wire_and_original_loan() {
    assert_eq!(encode_canonical_value(&value(), BOUNDS).unwrap(), RAW);
    let mut allocations = Vec::new();
    let mut work = 0;
    let decoded =
        decode_canonical_value_with_admission::<Expression>(RAW, BOUNDS, &mut |units, bytes| {
            work += units;
            if bytes > 0 {
                allocations.push(bytes);
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(decoded, value());
    assert!(
        work > RAW.len(),
        "nested visits and comparison consume the same loan"
    );
    assert!(
        allocations.contains(&std::mem::size_of::<Expression>()),
        "boxed payload is admitted before construction"
    );
    assert_eq!(
        worth_foundational::facade::ValueShapeDefinition::of::<Expression>()
            .inspect()
            .unwrap(),
        worth_foundational::facade::ValueShapeDepth::Recursive
    );
    assert!(matches!(
        crate::value_shape::ordinary_context::recursive::Expression::value_shape(),
        worth_foundational::facade::ValueShape::Recursive { binding: 0, .. }
    ));
    assert_eq!(
        decode_canonical_value::<Expression>(RAW, BOUNDS).unwrap(),
        value()
    );
}

#[test]
fn recursive_refusals_keep_cumulative_work_storage_and_fresh_retry() {
    let mut boxed = false;
    let mut stopped = false;
    let mut calls_after_stop = 0;
    let refused =
        decode_canonical_value_with_admission::<Expression>(RAW, BOUNDS, &mut |_, bytes| {
            if stopped {
                calls_after_stop += 1;
            }
            if bytes == std::mem::size_of::<Expression>() {
                boxed = true;
            }
            if bytes == "δ\n".len() {
                assert!(
                    boxed,
                    "the refusal must reach the boxed/pair payload, not setup"
                );
                stopped = true;
                return Err(worth_foundational::facade::ValueCodecDenial::Admission);
            }
            Ok(())
        });
    assert_eq!(
        refused,
        Err(worth_foundational::facade::ValueCodecDenial::Admission)
    );
    assert!(stopped);
    assert_eq!(calls_after_stop, 0);
    assert_eq!(
        decode_canonical_value_with_admission::<Expression>(RAW, BOUNDS, &mut |_, _| Ok(()))
            .unwrap(),
        value()
    );
    let mut boxed = false;
    assert_eq!(
        decode_canonical_value_with_admission::<Expression>(RAW, BOUNDS, &mut |work, bytes| {
            if bytes == std::mem::size_of::<Expression>() {
                boxed = true;
            } else if boxed && bytes == 0 && work > 0 {
                return Err(worth_foundational::facade::ValueCodecDenial::Admission);
            }
            Ok(())
        }),
        Err(worth_foundational::facade::ValueCodecDenial::Admission)
    );
    assert!(
        boxed,
        "work-only refusal must occur in the nested boxed seed"
    );
    assert_eq!(
        decode_canonical_value::<Expression>(
            RAW,
            worth_foundational::facade::ValueDecodeBounds::new(RAW.len() - 1, 64, 8, 24)
        ),
        Err(worth_foundational::facade::ValueCodecDenial::WireBytes)
    );
    assert!(decode_canonical_value::<Expression>(
        RAW,
        worth_foundational::facade::ValueDecodeBounds::new(4096, 64, 8, 3)
    )
    .is_err());
}

#[test]
fn tag_field_order_and_pair_arity_are_real_wire_constraints() {
    for raw in [
        r#"{"value":"x","kind":"text"}"#,
        r#"{"kind":"missing"}"#,
        r#"{"kind":"text","value":"x","kind":"empty"}"#,
        r#"{"kind":"text","value":"x","extra":0}"#,
        r#"{"kind":"text"}"#,
        r#"{"kind":"pairs","fields":[["left"]]}"#,
        r#"{"kind":"pairs","fields":[["left",{"kind":"empty"},{"bad":0}]]}"#,
    ] {
        assert!(
            decode_canonical_value::<Expression>(raw, BOUNDS).is_err(),
            "{raw}"
        );
    }
    assert_eq!(
        decode_canonical_value::<Expression>(r#" {"kind":"empty"}"#, BOUNDS),
        Err(worth_foundational::facade::ValueCodecDenial::NonCanonical)
    );
}

#[test]
fn pair_depth_refusal_keeps_its_cause_before_constructing_children() {
    let raw = r#"{"kind":"pairs","fields":[["left",{"kind":"empty"}]]}"#;
    let policy = worth_foundational::facade::ValueDecodeBounds::new(1024, 64, 8, 2);
    assert_eq!(
        decode_canonical_value::<Expression>(raw, policy),
        Err(worth_foundational::facade::ValueCodecDenial::Depth)
    );
    assert_eq!(
        decode_canonical_value_with_admission::<Expression>(raw, policy, &mut |_, _| Ok(())),
        Err(worth_foundational::facade::ValueCodecDenial::Depth)
    );
}
#[test]
fn portable_binders_exclude_rust_names_and_local_type_identity() {
    assert_eq!(
        crate::value_shape::ordinary_context::recursive::First::value_shape(),
        crate::value_shape::ordinary_context::recursive::Renamed::value_shape()
    );
}
#[test]
fn transparent_recursive_cycles_refuse_without_entering_a_seed() {
    assert_eq!(
        worth_foundational::facade::ValueShapeDefinition::of::<Uninhabited>().inspect(),
        Err(worth_foundational::facade::ValueCodecDenial::UnsupportedShape)
    );
    assert!(matches!(
        crate::value_shape::ordinary_context::recursive::Uninhabited::value_shape(),
        worth_foundational::facade::ValueShape::Recursive { .. }
    ));
    assert!(matches!(
        decode_canonical_value_with_admission::<Uninhabited>("null", BOUNDS, &mut |_, _| Ok(())),
        Err(worth_foundational::facade::ValueCodecDenial::UnsupportedShape)
    ));
}
