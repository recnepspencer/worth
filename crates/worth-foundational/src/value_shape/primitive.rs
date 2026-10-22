//! Fixed-width scalar decoding and length-admitted owned strings.
use super::{
    CanonicalValue, ValueCodecDenial as D, ValueDecodeAdmission, ValueDecodeBounds, ValuePrimitive,
    ValueShapeBody,
};
use serde::{
    de::{Error, Visitor},
    Deserialize,
};

macro_rules! fixed_scalar {
    ($($ty:ty => $shape:ident),* $(,)?) => {$ (
        impl CanonicalValue for $ty {
            const SUPPORTS_ADMISSION: bool = true;
                    const WIRE_NULLABLE: bool = matches!(ValuePrimitive::$shape, ValuePrimitive::Unit);
            const SHAPE_BODY: ValueShapeBody = { ValueShapeBody::Primitive(ValuePrimitive::$shape) };
            fn validate_bounds(&self, _: ValueDecodeBounds) -> Result<(), D> { Ok(()) }
            fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(decoder: Decoder, _: ValueDecodeBounds) -> Result<Self, Decoder::Error> {
                Self::deserialize(decoder)
            }
            fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(decoder: Decoder, bounds: ValueDecodeBounds, _: &mut ValueDecodeAdmission<'_>) -> Result<Self, Decoder::Error> {
                Self::deserialize_bounded(decoder, bounds)
            }
        }
    )*};
}
fixed_scalar!(bool => Bool, i8 => I8, i16 => I16, i32 => I32, i64 => I64,
    u8 => U8, u16 => U16, u32 => U32, u64 => U64, () => Unit);

impl CanonicalValue for String {
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = { ValueShapeBody::Primitive(ValuePrimitive::String) };
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), D> {
        bounds.check_string_bytes(self.len())
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        decode_string(decoder, bounds, None)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        decode_string(decoder, bounds, Some(admission))
    }
    fn deserialize_reporting<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        reporter: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        decode_string(decoder, bounds, Some(reporter))
    }
}

fn decode_string<'de, Decoder: serde::Deserializer<'de>>(
    decoder: Decoder,
    bounds: ValueDecodeBounds,
    admission: Option<&mut ValueDecodeAdmission<'_>>,
) -> Result<String, Decoder::Error> {
    struct BoundedString<'a, 'b>(ValueDecodeBounds, Option<&'a mut ValueDecodeAdmission<'b>>);
    impl Visitor<'_> for BoundedString<'_, '_> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a bounded string")
        }
        fn visit_str<E: Error>(mut self, value: &str) -> Result<String, E> {
            self.0
                .check_string_bytes(value.len())
                .map_err(|cause| super::value_decode_error::<E>(cause, self.1.as_deref_mut()))?;
            if let Some(admit) = self.1.as_mut() {
                admit.admit(value.len(), value.len()).map_err(E::custom)?;
            }
            let mut result = String::new();
            result.try_reserve_exact(value.len()).map_err(|_| {
                super::value_decode_error::<E>(D::Allocation, self.1.as_deref_mut())
            })?;
            result.push_str(value);
            Ok(result)
        }
        fn visit_string<E: Error>(mut self, value: String) -> Result<String, E> {
            value
                .validate_bounds(self.0)
                .map_err(|cause| super::value_decode_error::<E>(cause, self.1.as_deref_mut()))?;
            if let Some(admit) = self.1.as_mut() {
                admit
                    .admit(value.len(), value.capacity())
                    .map_err(E::custom)?;
            }
            Ok(value)
        }
    }
    decoder.deserialize_str(BoundedString(bounds, admission))
}
