//! Canonical JSON emitted with the value's shape and checked construction seed.
use super::{
    ValueCodecDenial as D, ValueDecodeAdmission, ValueDecodeBounds, ValueSeedDelegate, ValueShape,
    ValueShapeBody, ValueShapeDefinition,
};
use serde::{de::DeserializeSeed, Serialize};
use std::io::Write;

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

    /// Standalone construction with typed failure reporting, without native
    /// admission requirements. Opaque bounded-only owners retain their posture.
    fn deserialize_reporting<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        _reporter: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        Self::deserialize_bounded(decoder, bounds)
    }

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
/// guarded static grammar edges and optional domain representation policies.
/// Decoded owned strings and sequences use fallible reservations and the
/// original admission callback; no speculative parser storage quote is needed.
pub fn decode_canonical_value<Value: CanonicalValue>(
    raw: &str,
    bounds: ValueDecodeBounds,
) -> Result<Value, D> {
    super::admitted::decode_recording::<Value>(raw, bounds, &mut |_, _| Ok(()), false)
}

/// Validate structure first; the writer checks encoded bytes before reserving.
pub fn encode_canonical_value<Value: CanonicalValue>(
    value: &Value,
    bounds: ValueDecodeBounds,
) -> Result<String, D> {
    encode_delegated_canonical_value::<super::ValueDelegate<Value>>(value, bounds)
}

/// The same canonical encoder for an entry-owned delegate of a foreign value.
/// No wrapper value or second serialization is required.
pub fn encode_delegated_canonical_value<Delegate: ValueSeedDelegate>(
    value: &Delegate::Value,
    bounds: ValueDecodeBounds,
) -> Result<String, D> {
    ValueShapeDefinition::from_delegate::<Delegate>().inspect()?;
    Delegate::validate(value, bounds)?;
    let mut writer = BoundedJsonWriter {
        bytes: Vec::new(),
        maximum: bounds.maximum_wire_bytes(),
        denial: None,
    };
    let mut serializer = serde_json::Serializer::new(&mut writer);
    if value
        .serialize(serde_stacker::Serializer::new(&mut serializer))
        .is_err()
    {
        return Err(writer.denial.unwrap_or(D::Malformed));
    }
    if let Some(denial) = writer.denial {
        return Err(denial);
    }
    String::from_utf8(writer.bytes).map_err(|_| D::Malformed)
}

struct BoundedJsonWriter {
    bytes: Vec<u8>,
    maximum: Option<usize>,
    denial: Option<D>,
}
impl Write for BoundedJsonWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Some(denial) = self.denial {
            return Err(std::io::Error::other(denial));
        }
        let denial = if self
            .maximum
            .is_some_and(|maximum| bytes.len() > maximum.saturating_sub(self.bytes.len()))
        {
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
    fn reserve(&mut self, additional: usize) -> Result<(), D> {
        let required = self
            .bytes
            .len()
            .checked_add(additional)
            .ok_or(D::Allocation)?;
        if required > self.bytes.capacity() {
            let mut target = self.bytes.capacity().saturating_mul(2).max(required);
            if let Some(maximum) = self.maximum {
                target = target.min(maximum);
            }
            self.bytes
                .try_reserve_exact(target - self.bytes.len())
                .map_err(|_| D::Allocation)?;
        }
        Ok(())
    }
}
