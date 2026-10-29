// Neutral recursive tagged, named, boxed and fixed-product grammar.
pub(crate) use {worth_foundational::facade::decode_canonical_value, worth_foundational::facade::decode_canonical_value_with_admission, worth_foundational::facade::encode_canonical_value, worth_foundational::facade::CanonicalValue, worth_foundational::facade::ValueCodecDenial as D, worth_foundational::facade::ValueDecodeBounds, worth_foundational::facade::ValueShape, worth_foundational::facade::ValueShapeDefinition, worth_foundational::facade::ValueShapeDepth, worth_foundational::facade::WorthValue};

#[derive(Debug, PartialEq, WorthValue)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Expression {
    Empty,
    Text { value: String },
    Negate { operand: Box<Expression> },
    Group { members: Vec<Element> },
    Pairs { fields: Vec<(Label, Expression)> },
}
#[derive(Debug, PartialEq, WorthValue)]
pub(crate) struct Label(pub(crate) String);
#[derive(Debug, PartialEq, WorthValue)]
#[serde(deny_unknown_fields)]
pub(crate) struct Element {
    pub(crate) label: Label,
    pub(crate) value: Expression,
}

pub(crate) const BOUNDS: ValueDecodeBounds = worth_foundational::facade::ValueDecodeBounds::new(4096, 64, 8, 24);
pub(crate) const RAW: &str = r#"{"kind":"group","members":[{"label":"α","value":{"kind":"negate","operand":{"kind":"pairs","fields":[["left",{"kind":"text","value":"δ\n"}],["right",{"kind":"empty"}]]}}}]}"#;
pub(crate) fn value() -> Expression {
    crate::value_shape::ordinary_context::recursive::Expression::Group {
        members: vec![Element {
            label: Label("α".into()),
            value: crate::value_shape::ordinary_context::recursive::Expression::Negate {
                operand: Box::new(crate::value_shape::ordinary_context::recursive::Expression::Pairs {
                    fields: vec![
                        (
                            Label("left".into()),
                            crate::value_shape::ordinary_context::recursive::Expression::Text {
                                value: "δ\n".into(),
                            },
                        ),
                        (Label("right".into()), crate::value_shape::ordinary_context::recursive::Expression::Empty),
                    ],
                }),
            },
        }],
    }
}

#[derive(WorthValue)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum First {
    Leaf,
    Branch { children: Vec<First> },
}
#[derive(WorthValue)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Renamed {
    Leaf,
    Branch { children: Vec<Renamed> },
}

#[derive(serde::Serialize)]
pub(crate) enum Uninhabited {}
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