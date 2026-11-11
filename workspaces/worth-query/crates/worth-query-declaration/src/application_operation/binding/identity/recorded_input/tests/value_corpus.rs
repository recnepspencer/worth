//! Concrete grammar values and declared validation, without a runtime.
use crate::facade::application_schema::{
    ApplicationStructuredValueBinding, ApplicationValueValidationDenial,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(super) struct Corpus {
    signed: Vec<i128>,
    unsigned: Vec<u128>,
    widths: (i8, i16, i32, i64, u8, u16, u32, u64),
    floating: (f32, f64),
    text: String,
    character: char,
    nested: Option<Option<u64>>,
    absent: Option<u64>,
    unit: (),
    marker: Marker,
    pair: Pair,
    wrapped: Wrapped,
    variants: Vec<Variant>,
    map: BTreeMap<String, Vec<u64>>,
    bytes: BytePayload,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Marker;
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Pair(u32, bool);
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Wrapped(String);
#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Variant {
    Empty,
    New(u64),
    Tuple(u8, String),
    Fields { flag: bool, value: u32 },
}
#[derive(Debug, PartialEq)]
struct BytePayload(Vec<u8>);
impl Serialize for BytePayload {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}
impl<'de> Deserialize<'de> for BytePayload {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Bytes;
        impl<'de> serde::de::Visitor<'de> for Bytes {
            type Value = BytePayload;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("original byte payload")
            }
            fn visit_borrowed_bytes<E: serde::de::Error>(
                self,
                value: &'de [u8],
            ) -> Result<Self::Value, E> {
                Ok(BytePayload(value.to_vec()))
            }
            fn visit_bytes<E: serde::de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                Ok(BytePayload(value.to_vec()))
            }
        }
        deserializer.deserialize_byte_buf(Bytes)
    }
}
pub(super) fn corpus() -> Corpus {
    Corpus {
        signed: vec![i128::MIN, -17, 0, i128::MAX],
        unsigned: vec![0, 128, u128::MAX],
        widths: (
            i8::MIN,
            i16::MIN,
            i32::MIN,
            i64::MIN,
            u8::MAX,
            u16::MAX,
            u32::MAX,
            u64::MAX,
        ),
        floating: (-0.0, 1.625),
        text: "canonical\0: stream".into(),
        character: '\u{1f642}',
        nested: Some(Some(13)),
        absent: None,
        unit: (),
        marker: Marker,
        pair: Pair(19, true),
        wrapped: Wrapped("wrapped".into()),
        variants: vec![
            Variant::Empty,
            Variant::New(7),
            Variant::Tuple(3, "tuple".into()),
            Variant::Fields {
                flag: false,
                value: 8,
            },
        ],
        map: BTreeMap::from([("z".into(), vec![]), ("a".into(), vec![1, 2])]),
        bytes: BytePayload(vec![0, 1, 255]),
    }
}

pub(super) struct CorpusBinding;
impl ApplicationStructuredValueBinding for CorpusBinding {
    type Value = Corpus;
    const IDENTITY_NAME: &'static str = "worth.recorded-input.corpus.v1";
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

pub(super) struct ForeignBinding;
impl ApplicationStructuredValueBinding for ForeignBinding {
    type Value = Corpus;
    const IDENTITY_NAME: &'static str = "worth.recorded-input.foreign.v1";
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

pub(super) struct ScalarBinding;
impl ApplicationStructuredValueBinding for ScalarBinding {
    type Value = u64;
    const IDENTITY_NAME: &'static str = "worth.recorded-input.scalar.v1";
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

pub(super) struct CapturedScalarBinding;
impl ApplicationStructuredValueBinding for CapturedScalarBinding {
    type Value = u64;
    const IDENTITY_NAME: &'static str = "worth.recorded-input.captured-scalar.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

pub(super) struct MapBinding;
impl ApplicationStructuredValueBinding for MapBinding {
    type Value = BTreeMap<String, u64>;
    const IDENTITY_NAME: &'static str = "worth.recorded-input.map.v1";
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

pub(super) struct PositiveBinding;
impl ApplicationStructuredValueBinding for PositiveBinding {
    type Value = u64;
    const IDENTITY_NAME: &'static str = "worth.recorded-input.positive.v1";
    fn validate(value: &u64) -> Result<(), ApplicationValueValidationDenial> {
        if *value > 0 {
            Ok(())
        } else {
            Err(ApplicationValueValidationDenial::rejected(
                Self::IDENTITY,
                "positive value required",
            ))
        }
    }
}
