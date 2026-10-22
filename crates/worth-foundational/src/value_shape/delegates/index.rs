//! Platform indices use the existing u64 wire grammar with checked conversion.
use super::*;

pub struct UsizeDelegate;
impl ValueSeedDelegate for UsizeDelegate {
    type Value = usize;
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = <u64 as CanonicalValue>::SHAPE_BODY;
    fn validate(value: &usize, bounds: ValueDecodeBounds) -> Result<(), D> {
        let value = u64::try_from(*value).map_err(|_| D::Malformed)?;
        value.validate_bounds(bounds)
    }
    fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        mut admission: Option<&mut ValueDecodeAdmission<'_>>,
    ) -> Result<usize, Decoder::Error> {
        let value = ValueDelegate::<u64>::deserialize(decoder, bounds, admission.as_deref_mut())?;
        usize::try_from(value).map_err(|_| {
            super::super::value_decode_error::<Decoder::Error>(D::Malformed, admission)
        })
    }
}

impl CanonicalValue for usize {
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = UsizeDelegate::SHAPE_BODY;
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), D> {
        UsizeDelegate::validate(self, bounds)
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        UsizeDelegate::deserialize(decoder, bounds, None)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        UsizeDelegate::deserialize(decoder, bounds, Some(admission))
    }
    fn deserialize_reporting<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        reporter: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        UsizeDelegate::deserialize(decoder, bounds, Some(reporter))
    }
}
