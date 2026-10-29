



#[allow(unused_imports)]
pub(crate) use {};



pub(crate) use {worth_foundational::facade::decode_canonical_value, worth_foundational::facade::encode_canonical_value, worth_foundational::facade::CanonicalValue, worth_foundational::facade::ValueCodecDenial, worth_foundational::facade::ValueDecodeBounds, worth_foundational::facade::WorthValue};

#[derive(Clone, Debug, PartialEq, WorthValue)]
pub(crate) struct Packet {
    #[serde(rename = "selection")]
    pub(crate) value: Choice,
    pub(crate) children: Vec<Child>,
}
#[derive(Clone, Debug, PartialEq, WorthValue)]
pub(crate) enum Choice {
    Empty,
    #[serde(rename = "chosen")]
    Selected(Child),
}
#[derive(Clone, Debug, PartialEq, WorthValue)]
pub(crate) struct Child {
    pub(crate) count: i32,
    pub(crate) text: String,
    pub(crate) present: Option<bool>,
}

pub(crate) const BOUNDS: ValueDecodeBounds = worth_foundational::facade::ValueDecodeBounds::new(1024, 32, 2, 12);
pub(crate) const RAW: &str = r#"{"selection":{"chosen":{"count":-17,"text":"nested","present":false}},"children":[{"count":9,"text":"δ","present":null}]}"#;

pub(crate) fn packet() -> Packet {
    Packet {
        value: crate::value_shape::ordinary_context::Choice::Selected(Child {
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

#[derive(Debug, PartialEq, WorthValue)]
pub(crate) enum UnitVariant {
    Choice,
}
#[derive(Debug, PartialEq, WorthValue)]
pub(crate) enum UnitPayload {
    Choice(()),
}