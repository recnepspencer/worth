//! A bounded JSON wire contract emitted with the value's shape and seed.
use super::{
    ValueCodecDenial as D, ValueDecodeBounds, ValueShape, ValueShapeBody, ValueShapeDefinition,
};
use serde::{de::DeserializeSeed, Serialize};
use std::io::Write;

/// Borrowed admission callback; it does not itself establish request authority.
/// The native owner supplies that authority and retains granted reservations.
/// Bytes are newly requested storage, not wire size or retained ownership.
pub type ValueDecodeAdmission<'a> = dyn FnMut(usize, usize) -> Result<(), D> + 'a;

/// Implementations must describe and emit the same complete wire grammar.
/// Derive `WorthValue` for ordinary structs/enums. Opaque domain owners implement
/// this contract only with their own bounded seed and admitted constructor.
/// No `DeserializeOwned` fallback participates in this contract.
pub trait CanonicalValue: Serialize + Sized + 'static {
    /// This owner's seed, independently of its children's support. The complete
    /// static closure is checked by installation and admitted decoding.
    const SUPPORTS_ADMISSION: bool = false;
    /// Whether this wire grammar can emit JSON null. Optional payloads must
    /// exclude null so absence cannot erase a present value's meaning.
    const WIRE_NULLABLE: bool;
    const SHAPE_BODY: ValueShapeBody;
    fn value_shape() -> ValueShape {
        ValueShapeDefinition::of::<Self>().materialize()
    }
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), D>;
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error>;

    /// Opaque owners must admit their variable work/storage, including custom
    /// diagnostics beyond the built-in JSON machinery's bounded diagnostics.
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        _decoder: Decoder,
        _bounds: ValueDecodeBounds,
        _admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        Err(serde::de::Error::custom(D::UnsupportedAdmission))
    }
}

/// A bounded grammar seed. `new` enforces representation bounds only;
/// `with_admission` also passes the caller's request loan through nested seeds.
/// Whole-carrier decoding additionally checks input and canonical form.
pub struct CanonicalValueSeed<'a, 'request, Value> {
    seed: super::DelegatedValueSeed<'a, 'request, super::ValueDelegate<Value>>,
}
impl<'a, 'request, Value> CanonicalValueSeed<'a, 'request, Value> {
    pub fn new(bounds: ValueDecodeBounds) -> Self {
        Self {
            seed: super::DelegatedValueSeed::new(bounds, None),
        }
    }
    /// The same request loan is reborrowed by all nested seeds.
    pub fn with_admission(
        bounds: ValueDecodeBounds,
        admission: &'a mut ValueDecodeAdmission<'request>,
    ) -> Self {
        Self {
            seed: super::DelegatedValueSeed::new(bounds, Some(admission)),
        }
    }
    pub fn with_optional_admission(
        bounds: ValueDecodeBounds,
        admission: Option<&'a mut ValueDecodeAdmission<'request>>,
    ) -> Self {
        Self {
            seed: super::DelegatedValueSeed::new(bounds, admission),
        }
    }
}
impl<'de, Value: CanonicalValue> DeserializeSeed<'de> for CanonicalValueSeed<'_, '_, Value> {
    type Value = Value;
    fn deserialize<Decoder: serde::Deserializer<'de>>(
        self,
        decoder: Decoder,
    ) -> Result<Value, Decoder::Error> {
        self.seed.deserialize(decoder)
    }
}

/// O(grammar visits * active type depth + wire bytes + value nodes), with
/// finite input/container/value-depth bounds and guarded static grammar edges.
/// JSON escape scratch is bounded by the admitted input bytes; decoded strings
/// and sequences are admitted before their owned storage grows.
pub fn decode_canonical_value<Value: CanonicalValue>(
    raw: &str,
    bounds: ValueDecodeBounds,
) -> Result<Value, D> {
    if raw.len() > bounds.maximum_wire_bytes() {
        return Err(D::WireBytes);
    }
    ValueShapeDefinition::of::<Value>().inspect()?;
    let mut decoder = serde_json::Deserializer::from_str(raw);
    let value = CanonicalValueSeed::<Value>::new(bounds)
        .deserialize(&mut decoder)
        .map_err(|_| D::Malformed)?;
    decoder.end().map_err(|_| D::Malformed)?;
    value.validate_bounds(bounds)?;
    if encode_canonical_value(&value, bounds)? != raw {
        return Err(D::NonCanonical);
    }
    Ok(value)
}

/// Validate structure first; the writer checks encoded bytes before reserving.
pub fn encode_canonical_value<Value: CanonicalValue>(
    value: &Value,
    bounds: ValueDecodeBounds,
) -> Result<String, D> {
    ValueShapeDefinition::of::<Value>().inspect()?;
    value.validate_bounds(bounds)?;
    let mut writer = BoundedJsonWriter {
        bytes: Vec::new(),
        maximum: bounds.maximum_wire_bytes(),
        denial: None,
    };
    if serde_json::to_writer(&mut writer, value).is_err() {
        return Err(writer.denial.unwrap_or(D::Malformed));
    }
    if let Some(denial) = writer.denial {
        return Err(denial);
    }
    String::from_utf8(writer.bytes).map_err(|_| D::Malformed)
}

struct BoundedJsonWriter {
    bytes: Vec<u8>,
    maximum: usize,
    denial: Option<D>,
}
impl Write for BoundedJsonWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Some(denial) = self.denial {
            return Err(std::io::Error::other(denial));
        }
        let denial = if bytes.len() > self.maximum.saturating_sub(self.bytes.len()) {
            Some(D::WireBytes)
        } else if self.reserve(bytes.len()).is_err() {
            Some(D::Allocation)
        } else {
            None
        };
        if let Some(denial) = denial {
            self.denial = Some(denial);
            return Err(std::io::Error::other(denial));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl BoundedJsonWriter {
    fn reserve(&mut self, additional: usize) -> Result<(), std::collections::TryReserveError> {
        let required = self.bytes.len() + additional;
        if required > self.bytes.capacity() {
            let target = self
                .bytes
                .capacity()
                .saturating_mul(2)
                .max(required)
                .min(self.maximum);
            self.bytes.try_reserve_exact(target - self.bytes.len())?;
        }
        Ok(())
    }
}
