//! A bounded JSON wire contract emitted with the value's shape and seed.
use super::{ValueCodecDenial as D, ValueDecodeBounds, ValueShape};
use serde::{de::DeserializeSeed, Serialize};
use std::{io::Write, marker::PhantomData};

/// Borrowed admission callback; it does not itself establish request authority.
/// The native owner supplies that authority and retains granted reservations.
/// Bytes are newly requested storage, not wire size or retained ownership.
pub type ValueDecodeAdmission<'a> = dyn FnMut(usize, usize) -> Result<(), D> + 'a;

/// Implementations must describe and emit the same complete wire grammar.
/// Derive `WorthValue` for ordinary structs/enums. Opaque domain owners implement
/// this contract only with their own bounded seed and admitted constructor.
/// No `DeserializeOwned` fallback participates in this contract.
pub trait CanonicalValue: Serialize + Sized {
    /// Opaque owners must explicitly supply an admitted seed; no fallback.
    const SUPPORTS_ADMISSION: bool = false;
    /// Finite structural depth. Recursive type cycles are compile-time errors.
    const SHAPE_DEPTH: usize;
    /// Whether this wire grammar can emit JSON null. Optional payloads must
    /// exclude null so absence cannot erase a present value's meaning.
    const WIRE_NULLABLE: bool;
    fn value_shape() -> ValueShape;
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

/// A finite grammar seed. `new` enforces representation bounds only;
/// `with_admission` also passes the caller's request loan through nested seeds.
/// Whole-carrier decoding additionally checks input and canonical form.
pub struct CanonicalValueSeed<'a, 'request, Value> {
    bounds: ValueDecodeBounds,
    admission: Option<&'a mut ValueDecodeAdmission<'request>>,
    value: PhantomData<fn() -> Value>,
}
impl<'a, 'request, Value> CanonicalValueSeed<'a, 'request, Value> {
    pub const fn new(bounds: ValueDecodeBounds) -> Self {
        Self {
            bounds,
            admission: None,
            value: PhantomData,
        }
    }

    /// The same request loan is reborrowed by all nested seeds.
    pub fn with_admission(
        bounds: ValueDecodeBounds,
        admission: &'a mut ValueDecodeAdmission<'request>,
    ) -> Self {
        Self {
            bounds,
            admission: Some(admission),
            value: PhantomData,
        }
    }

    /// Shared generated visitor plumbing for standalone and admitted decoding.
    pub fn with_optional_admission(
        bounds: ValueDecodeBounds,
        admission: Option<&'a mut ValueDecodeAdmission<'request>>,
    ) -> Self {
        Self {
            bounds,
            admission,
            value: PhantomData,
        }
    }
}
impl<'de, Value: CanonicalValue> DeserializeSeed<'de> for CanonicalValueSeed<'_, '_, Value> {
    type Value = Value;
    fn deserialize<Decoder: serde::Deserializer<'de>>(
        self,
        decoder: Decoder,
    ) -> Result<Value, Decoder::Error> {
        match self.admission {
            Some(admission) => {
                // Decode, bounds validation and structural re-encoding traversal.
                admission(3, 0).map_err(serde::de::Error::custom)?;
                Value::deserialize_admitted(decoder, self.bounds, admission)
            }
            None => Value::deserialize_bounded(decoder, self.bounds),
        }
    }
}

/// O(wire bytes + value nodes), with finite input, container and nesting bounds.
/// JSON escape scratch is bounded by the admitted input bytes; decoded strings
/// and sequences are admitted before their owned storage grows.
pub fn decode_canonical_value<Value: CanonicalValue>(
    raw: &str,
    bounds: ValueDecodeBounds,
) -> Result<Value, D> {
    if raw.len() > bounds.maximum_wire_bytes() {
        return Err(D::WireBytes);
    }
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
