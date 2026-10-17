//! Bounded recursive sequence and optional value construction.
use super::{
    CanonicalValue, CanonicalValueSeed, ValueCodecDenial as D, ValueDecodeAdmission,
    ValueDecodeBounds, ValueShape,
};
use serde::de::{DeserializeSeed, Error, SeqAccess, Visitor};
use std::marker::PhantomData;

impl<Value: CanonicalValue> CanonicalValue for Vec<Value> {
    const SUPPORTS_ADMISSION: bool = Value::SUPPORTS_ADMISSION;
    const SHAPE_DEPTH: usize = 1 + Value::SHAPE_DEPTH;
    const WIRE_NULLABLE: bool = false;
    fn value_shape() -> ValueShape {
        ValueShape::Sequence(Box::new(Value::value_shape()))
    }
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), D> {
        if self.len() > bounds.maximum_sequence_length() {
            return Err(D::SequenceLength);
        }
        let child = bounds.descend()?;
        for value in self {
            value.validate_bounds(child)?;
        }
        Ok(())
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        decode_sequence(decoder, bounds, None)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        decode_sequence(decoder, bounds, Some(admission))
    }
}

fn decode_sequence<'de, Value: CanonicalValue, Decoder: serde::Deserializer<'de>>(
    decoder: Decoder,
    bounds: ValueDecodeBounds,
    admission: Option<&mut ValueDecodeAdmission<'_>>,
) -> Result<Vec<Value>, Decoder::Error> {
    let child = bounds.descend().map_err(Decoder::Error::custom)?;
    struct Sequence<'a, 'b, Value>(
        ValueDecodeBounds,
        usize,
        PhantomData<Value>,
        Option<&'a mut ValueDecodeAdmission<'b>>,
    );
    impl<'de, Value: CanonicalValue> Visitor<'de> for Sequence<'_, '_, Value> {
        type Value = Vec<Value>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a bounded sequence")
        }
        fn visit_seq<Access: SeqAccess<'de>>(
            mut self,
            mut access: Access,
        ) -> Result<Self::Value, Access::Error> {
            let mut values = Vec::new();
            loop {
                if values.len() == self.1 {
                    // The rejecting seed detects an extra item before constructing it.
                    access.next_element_seed(RejectAdditionalElement)?;
                    return Ok(values);
                }
                let Some(value) = access.next_element_seed(
                    CanonicalValueSeed::<Value>::with_optional_admission(
                        self.0,
                        self.3.as_deref_mut(),
                    ),
                )?
                else {
                    return Ok(values);
                };
                if values.len() == values.capacity() {
                    let capacity = values.capacity().saturating_mul(2).max(4).min(self.1);
                    if let Some(admit) = self.3.as_deref_mut() {
                        let bytes = capacity
                            .checked_mul(std::mem::size_of::<Value>())
                            .ok_or_else(|| Access::Error::custom(D::Admission))?;
                        // The previous allocation remains charged during growth.
                        admit(1, bytes).map_err(Access::Error::custom)?;
                    }
                    values
                        .try_reserve_exact(capacity - values.len())
                        .map_err(|_| Access::Error::custom(D::Allocation))?;
                }
                values.push(value);
            }
        }
    }
    decoder.deserialize_seq(Sequence(
        child,
        bounds.maximum_sequence_length(),
        PhantomData,
        admission,
    ))
}

struct RejectAdditionalElement;
impl<'de> DeserializeSeed<'de> for RejectAdditionalElement {
    type Value = ();
    fn deserialize<Decoder: serde::Deserializer<'de>>(
        self,
        _: Decoder,
    ) -> Result<(), Decoder::Error> {
        Err(Decoder::Error::custom(D::SequenceLength))
    }
}

impl<Value: CanonicalValue> CanonicalValue for Option<Value> {
    const SUPPORTS_ADMISSION: bool = Value::SUPPORTS_ADMISSION;
    const SHAPE_DEPTH: usize = {
        assert!(!Value::WIRE_NULLABLE, "canonical Option payload can encode null; use a named externally tagged enum to preserve presence");
        1 + Value::SHAPE_DEPTH
    };
    const WIRE_NULLABLE: bool = true;
    fn value_shape() -> ValueShape {
        require_nonnullable_optional_payload::<Value>();
        ValueShape::Optional(Box::new(Value::value_shape()))
    }
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), D> {
        require_nonnullable_optional_payload::<Value>();
        let child = bounds.descend()?;
        if let Some(value) = self {
            value.validate_bounds(child)?;
        }
        Ok(())
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        decode_optional(decoder, bounds, None)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        decode_optional(decoder, bounds, Some(admission))
    }
}

fn decode_optional<'de, Value: CanonicalValue, Decoder: serde::Deserializer<'de>>(
    decoder: Decoder,
    bounds: ValueDecodeBounds,
    admission: Option<&mut ValueDecodeAdmission<'_>>,
) -> Result<Option<Value>, Decoder::Error> {
    require_nonnullable_optional_payload::<Value>();
    let child = bounds.descend().map_err(Decoder::Error::custom)?;
    struct Optional<'a, 'b, Value>(
        ValueDecodeBounds,
        PhantomData<Value>,
        Option<&'a mut ValueDecodeAdmission<'b>>,
    );
    impl<'de, Value: CanonicalValue> Visitor<'de> for Optional<'_, '_, Value> {
        type Value = Option<Value>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("an optional bounded value")
        }
        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<Decoder: serde::Deserializer<'de>>(
            self,
            decoder: Decoder,
        ) -> Result<Self::Value, Decoder::Error> {
            CanonicalValueSeed::<Value>::with_optional_admission(self.0, self.2)
                .deserialize(decoder)
                .map(Some)
        }
    }
    decoder.deserialize_option(Optional(child, PhantomData, admission))
}

fn require_nonnullable_optional_payload<Value: CanonicalValue>() {
    const {
        assert!(!Value::WIRE_NULLABLE, "canonical Option payload can encode null; use a named externally tagged enum to preserve presence");
    }
}
