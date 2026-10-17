//! Bounded recursive sequence and optional value construction.
use super::{
    CanonicalValue, CanonicalValueSeed, ValueCodecDenial as D, ValueDecodeBounds, ValueShape,
};
use serde::de::{DeserializeSeed, Error, SeqAccess, Visitor};
use std::marker::PhantomData;

impl<Value: CanonicalValue> CanonicalValue for Vec<Value> {
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
        let child = bounds.descend().map_err(Decoder::Error::custom)?;
        struct Sequence<Value>(ValueDecodeBounds, usize, PhantomData<Value>);
        impl<'de, Value: CanonicalValue> Visitor<'de> for Sequence<Value> {
            type Value = Vec<Value>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a bounded sequence")
            }
            fn visit_seq<Access: SeqAccess<'de>>(
                self,
                mut access: Access,
            ) -> Result<Self::Value, Access::Error> {
                let mut values = Vec::new();
                loop {
                    if values.len() == self.1 {
                        // The rejecting seed detects an extra item before constructing it.
                        access.next_element_seed(RejectAdditionalElement)?;
                        return Ok(values);
                    }
                    let Some(value) =
                        access.next_element_seed(CanonicalValueSeed::<Value>::new(self.0))?
                    else {
                        return Ok(values);
                    };
                    if values.len() == values.capacity() {
                        let capacity = values.capacity().saturating_mul(2).max(4).min(self.1);
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
        ))
    }
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
        require_nonnullable_optional_payload::<Value>();
        let child = bounds.descend().map_err(Decoder::Error::custom)?;
        struct Optional<Value>(ValueDecodeBounds, PhantomData<Value>);
        impl<'de, Value: CanonicalValue> Visitor<'de> for Optional<Value> {
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
                Value::deserialize_bounded(decoder, self.0).map(Some)
            }
        }
        decoder.deserialize_option(Optional(child, PhantomData))
    }
}

fn require_nonnullable_optional_payload<Value: CanonicalValue>() {
    const {
        assert!(!Value::WIRE_NULLABLE, "canonical Option payload can encode null; use a named externally tagged enum to preserve presence");
    }
}
