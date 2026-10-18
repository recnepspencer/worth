//! Mechanical seeds for syntactically declared container/product types.
//! A tuple delegate does not implement a blanket wire or residency trait on tuples.
mod boxed;
mod pair;
pub use super::containers::{OptionalDelegate, SequenceDelegate};
use super::{
    CanonicalValue, ValueCodecDenial as D, ValueDecodeAdmission, ValueDecodeBounds, ValueShapeBody,
};
pub use boxed::BoxDelegate;
pub use pair::PairDelegate;
use serde::{de::DeserializeSeed, Serialize};
use std::marker::PhantomData;

pub trait ValueSeedDelegate: 'static + Sized {
    type Value: Serialize;
    const SUPPORTS_ADMISSION: bool;
    const WIRE_NULLABLE: bool;
    const SHAPE_BODY: ValueShapeBody;
    const EDGE: super::ValueShapeEdge = super::ValueShapeEdge::delegate::<Self>();
    fn validate(value: &Self::Value, bounds: ValueDecodeBounds) -> Result<(), D>;
    fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: Option<&mut ValueDecodeAdmission<'_>>,
    ) -> Result<Self::Value, Decoder::Error>;
}

pub struct ValueDelegate<Value>(PhantomData<fn() -> Value>);
impl<Value: CanonicalValue> ValueSeedDelegate for ValueDelegate<Value> {
    type Value = Value;
    const SUPPORTS_ADMISSION: bool = Value::SUPPORTS_ADMISSION;
    const WIRE_NULLABLE: bool = Value::WIRE_NULLABLE;
    const SHAPE_BODY: ValueShapeBody = { Value::SHAPE_BODY };
    const EDGE: super::ValueShapeEdge = super::ValueShapeEdge::value::<Value>();
    fn validate(value: &Value, bounds: ValueDecodeBounds) -> Result<(), D> {
        value.validate_bounds(bounds)
    }
    fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: Option<&mut ValueDecodeAdmission<'_>>,
    ) -> Result<Value, Decoder::Error> {
        match admission {
            Some(admission) => Value::deserialize_admitted(decoder, bounds, admission),
            None => Value::deserialize_bounded(decoder, bounds),
        }
    }
}

pub struct DelegatedValueSeed<'loan, 'request, Delegate> {
    bounds: ValueDecodeBounds,
    admission: Option<&'loan mut ValueDecodeAdmission<'request>>,
    delegate: PhantomData<fn() -> Delegate>,
}
impl<'loan, 'request, Delegate> DelegatedValueSeed<'loan, 'request, Delegate> {
    pub fn new(
        bounds: ValueDecodeBounds,
        admission: Option<&'loan mut ValueDecodeAdmission<'request>>,
    ) -> Self {
        Self {
            bounds,
            admission,
            delegate: PhantomData,
        }
    }
}
impl<'de, Delegate: ValueSeedDelegate> DeserializeSeed<'de>
    for DelegatedValueSeed<'_, '_, Delegate>
{
    type Value = Delegate::Value;
    fn deserialize<Decoder: serde::Deserializer<'de>>(
        self,
        decoder: Decoder,
    ) -> Result<Self::Value, Decoder::Error> {
        if let Some(admission) = self.admission {
            admission(3, 0).map_err(serde::de::Error::custom)?;
            Delegate::deserialize(decoder, self.bounds, Some(admission))
        } else {
            Delegate::deserialize(decoder, self.bounds, None)
        }
    }
}
