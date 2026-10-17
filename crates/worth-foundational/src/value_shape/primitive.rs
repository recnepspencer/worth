//! Fixed-width scalar decoding and length-admitted owned strings.
use super::{
    CanonicalValue, ValueCodecDenial as D, ValueDecodeAdmission, ValueDecodeBounds, ValuePrimitive,
    ValueShape,
};
use serde::{
    de::{Error, Visitor},
    Deserialize,
};

macro_rules! fixed_scalar {
    ($($ty:ty => $shape:ident),* $(,)?) => {$ (
        impl CanonicalValue for $ty {
            const SUPPORTS_ADMISSION: bool = true;
            const SHAPE_DEPTH: usize = 0;
            const WIRE_NULLABLE: bool = matches!(ValuePrimitive::$shape, ValuePrimitive::Unit);
            fn value_shape() -> ValueShape { ValueShape::Primitive(ValuePrimitive::$shape) }
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
    const SHAPE_DEPTH: usize = 0;
    const WIRE_NULLABLE: bool = false;
    fn value_shape() -> ValueShape {
        ValueShape::Primitive(ValuePrimitive::String)
    }
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), D> {
        if self.len() > bounds.maximum_string_bytes() {
            Err(D::StringBytes)
        } else {
            Ok(())
        }
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
            if value.len() > self.0.maximum_string_bytes() {
                return Err(E::custom(D::StringBytes));
            }
            if let Some(admit) = self.1.as_mut() {
                admit(value.len(), value.len()).map_err(E::custom)?;
            }
            let mut result = String::new();
            result
                .try_reserve_exact(value.len())
                .map_err(|_| E::custom(D::Allocation))?;
            result.push_str(value);
            Ok(result)
        }
        fn visit_string<E: Error>(mut self, value: String) -> Result<String, E> {
            value.validate_bounds(self.0).map_err(E::custom)?;
            if let Some(admit) = self.1.as_mut() {
                admit(value.len(), value.capacity()).map_err(E::custom)?;
            }
            Ok(value)
        }
    }
    decoder.deserialize_str(BoundedString(bounds, admission))
}
