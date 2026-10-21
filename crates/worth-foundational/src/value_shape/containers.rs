//! Bounded recursive sequence and optional value construction.
use super::delegates::{DelegatedValueSeed, ValueDelegate, ValueSeedDelegate};
use super::{
    CanonicalValue, ValueCodecDenial as D, ValueDecodeAdmission, ValueDecodeBounds, ValueShapeBody,
};
use serde::de::{DeserializeSeed, Error, SeqAccess, Visitor};
use std::marker::PhantomData;

pub struct SequenceDelegate<Child>(PhantomData<fn() -> Child>);
impl<Child: ValueSeedDelegate> ValueSeedDelegate for SequenceDelegate<Child> {
    type Value = Vec<Child::Value>;
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = { ValueShapeBody::Sequence(Child::EDGE) };
    fn validate(value: &Self::Value, bounds: ValueDecodeBounds) -> Result<(), D> {
        if value.len() > bounds.maximum_sequence_length() {
            return Err(D::SequenceLength);
        }
        let child = bounds.descend()?;
        for value in value {
            Child::validate(value, child)?;
        }
        Ok(())
    }
    fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: Option<&mut ValueDecodeAdmission<'_>>,
    ) -> Result<Self::Value, Decoder::Error> {
        decode_sequence::<Child, Decoder>(decoder, bounds, admission)
    }
}
impl<Value: CanonicalValue> CanonicalValue for Vec<Value> {
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = { SequenceDelegate::<ValueDelegate<Value>>::SHAPE_BODY };
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), D> {
        SequenceDelegate::<ValueDelegate<Value>>::validate(self, bounds)
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        decode_sequence::<ValueDelegate<Value>, Decoder>(decoder, bounds, None)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        decode_sequence::<ValueDelegate<Value>, Decoder>(decoder, bounds, Some(admission))
    }
}

fn decode_sequence<'de, Value: ValueSeedDelegate, Decoder: serde::Deserializer<'de>>(
    decoder: Decoder,
    bounds: ValueDecodeBounds,
    admission: Option<&mut ValueDecodeAdmission<'_>>,
) -> Result<Vec<Value::Value>, Decoder::Error> {
    let child = bounds.descend().map_err(Decoder::Error::custom)?;
    struct Sequence<'a, 'b, Value>(
        ValueDecodeBounds,
        usize,
        PhantomData<Value>,
        Option<&'a mut ValueDecodeAdmission<'b>>,
    );
    impl<'de, Value: ValueSeedDelegate> Visitor<'de> for Sequence<'_, '_, Value> {
        type Value = Vec<Value::Value>;
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
                let Some(value) = access.next_element_seed(DelegatedValueSeed::<Value>::new(
                    self.0,
                    self.3.as_deref_mut(),
                ))?
                else {
                    return Ok(values);
                };
                if values.len() == values.capacity() {
                    let capacity = values.capacity().saturating_mul(2).max(4).min(self.1);
                    if let Some(admit) = self.3.as_deref_mut() {
                        let bytes = capacity
                            .checked_mul(std::mem::size_of::<Value::Value>())
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
    decoder.deserialize_seq(Sequence::<Value>(
        child,
        bounds.maximum_sequence_length(),
        PhantomData,
        admission,
    ))
}

pub(super) struct RejectAdditionalElement;
impl<'de> DeserializeSeed<'de> for RejectAdditionalElement {
    type Value = ();
    fn deserialize<Decoder: serde::Deserializer<'de>>(
        self,
        _: Decoder,
    ) -> Result<(), Decoder::Error> {
        Err(Decoder::Error::custom(D::SequenceLength))
    }
}

pub struct OptionalDelegate<Child>(PhantomData<fn() -> Child>);
impl<Child: ValueSeedDelegate> ValueSeedDelegate for OptionalDelegate<Child> {
    type Value = Option<Child::Value>;
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = true;
    const SHAPE_BODY: ValueShapeBody = {
        require_nonnullable_optional_payload::<Child>();
        ValueShapeBody::Optional(Child::EDGE)
    };
    fn validate(value: &Self::Value, bounds: ValueDecodeBounds) -> Result<(), D> {
        require_nonnullable_optional_payload::<Child>();
        let child = bounds.descend()?;
        if let Some(value) = value {
            Child::validate(value, child)?;
        }
        Ok(())
    }
    fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: Option<&mut ValueDecodeAdmission<'_>>,
    ) -> Result<Self::Value, Decoder::Error> {
        decode_optional::<Child, Decoder>(decoder, bounds, admission)
    }
}
impl<Value: CanonicalValue> CanonicalValue for Option<Value> {
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = true;
    const SHAPE_BODY: ValueShapeBody = { OptionalDelegate::<ValueDelegate<Value>>::SHAPE_BODY };
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), D> {
        OptionalDelegate::<ValueDelegate<Value>>::validate(self, bounds)
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        decode_optional::<ValueDelegate<Value>, Decoder>(decoder, bounds, None)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        decode_optional::<ValueDelegate<Value>, Decoder>(decoder, bounds, Some(admission))
    }
}

fn decode_optional<'de, Value: ValueSeedDelegate, Decoder: serde::Deserializer<'de>>(
    decoder: Decoder,
    bounds: ValueDecodeBounds,
    admission: Option<&mut ValueDecodeAdmission<'_>>,
) -> Result<Option<Value::Value>, Decoder::Error> {
    require_nonnullable_optional_payload::<Value>();
    let child = bounds.descend().map_err(Decoder::Error::custom)?;
    struct Optional<'a, 'b, Value>(
        ValueDecodeBounds,
        PhantomData<Value>,
        Option<&'a mut ValueDecodeAdmission<'b>>,
    );
    impl<'de, Value: ValueSeedDelegate> Visitor<'de> for Optional<'_, '_, Value> {
        type Value = Option<Value::Value>;
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
            DelegatedValueSeed::<Value>::new(self.0, self.2)
                .deserialize(decoder)
                .map(Some)
        }
    }
    decoder.deserialize_option(Optional::<Value>(child, PhantomData, admission))
}

const fn require_nonnullable_optional_payload<Value: ValueSeedDelegate>() {
    const {
        assert!(!Value::WIRE_NULLABLE, "canonical Option payload can encode null; use a named externally tagged enum to preserve presence");
    }
}
