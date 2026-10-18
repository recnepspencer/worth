//! Neutral recursive tagged, named, boxed and fixed-product grammar.
use worth_foundational::facade::{
    decode_canonical_value, decode_canonical_value_with_admission, encode_canonical_value,
    CanonicalValue, ValueCodecDenial as D, ValueDecodeBounds, ValueShape, ValueShapeDefinition,
    ValueShapeDepth, WorthValue,
};

#[derive(Debug, PartialEq, WorthValue)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Expression {
    Empty,
    Text { value: String },
    Negate { operand: Box<Expression> },
    Group { members: Vec<Element> },
    Pairs { fields: Vec<(Label, Expression)> },
}
#[derive(Debug, PartialEq, WorthValue)]
struct Label(String);
#[derive(Debug, PartialEq, WorthValue)]
#[serde(deny_unknown_fields)]
struct Element {
    label: Label,
    value: Expression,
}

const BOUNDS: ValueDecodeBounds = ValueDecodeBounds::new(4096, 64, 8, 24);
const RAW: &str = r#"{"kind":"group","members":[{"label":"α","value":{"kind":"negate","operand":{"kind":"pairs","fields":[["left",{"kind":"text","value":"δ\n"}],["right",{"kind":"empty"}]]}}}]}"#;
fn value() -> Expression {
    Expression::Group {
        members: vec![Element {
            label: Label("α".into()),
            value: Expression::Negate {
                operand: Box::new(Expression::Pairs {
                    fields: vec![
                        (
                            Label("left".into()),
                            Expression::Text {
                                value: "δ\n".into(),
                            },
                        ),
                        (Label("right".into()), Expression::Empty),
                    ],
                }),
            },
        }],
    }
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
        ValueShapeDefinition::of::<Expression>().inspect().unwrap(),
        ValueShapeDepth::Recursive
    );
    assert!(matches!(
        Expression::value_shape(),
        ValueShape::Recursive { binding: 0, .. }
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
                return Err(D::Admission);
            }
            Ok(())
        });
    assert_eq!(refused, Err(D::Admission));
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
                return Err(D::Admission);
            }
            Ok(())
        }),
        Err(D::Admission)
    );
    assert!(
        boxed,
        "work-only refusal must occur in the nested boxed seed"
    );
    assert_eq!(
        decode_canonical_value::<Expression>(RAW, ValueDecodeBounds::new(RAW.len() - 1, 64, 8, 24)),
        Err(D::WireBytes)
    );
    assert!(
        decode_canonical_value::<Expression>(RAW, ValueDecodeBounds::new(4096, 64, 8, 3)).is_err()
    );
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
        Err(D::NonCanonical)
    );
}

#[derive(WorthValue)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum First {
    Leaf,
    Branch { children: Vec<First> },
}
#[derive(WorthValue)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Renamed {
    Leaf,
    Branch { children: Vec<Renamed> },
}
#[test]
fn portable_binders_exclude_rust_names_and_local_type_identity() {
    assert_eq!(First::value_shape(), Renamed::value_shape());
}

#[derive(serde::Serialize)]
enum Uninhabited {}
impl CanonicalValue for Uninhabited {
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: worth_foundational::facade::ValueShapeBody = {
        worth_foundational::facade::ValueShapeBody::Transparent(
            worth_foundational::facade::ValueShapeEdge::value::<Self>(),
        )
    };
    fn validate_bounds(&self, _: ValueDecodeBounds) -> Result<(), D> {
        match *self {}
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        _: Decoder,
        _: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        panic!("unproductive recursion must refuse before the seed")
    }
}
#[test]
fn transparent_recursive_cycles_refuse_without_entering_a_seed() {
    assert_eq!(
        ValueShapeDefinition::of::<Uninhabited>().inspect(),
        Err(D::UnsupportedShape)
    );
    assert!(matches!(
        Uninhabited::value_shape(),
        ValueShape::Recursive { .. }
    ));
    assert!(matches!(
        decode_canonical_value_with_admission::<Uninhabited>("null", BOUNDS, &mut |_, _| Ok(())),
        Err(D::UnsupportedShape)
    ));
}
