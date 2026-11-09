//! Serializing a value does not prove that its custom seed preserves that value.
use super::{decode_stored_canonical_value, encode_stored_canonical_value};
use crate::value_shape::{CanonicalValue, ValueCodecDenial, ValueDecodeBounds, ValueShapeBody};

#[derive(Debug, PartialEq, serde::Serialize)]
#[serde(transparent)]
struct Normalizing(u64);

impl CanonicalValue for Normalizing {
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = <u64 as CanonicalValue>::SHAPE_BODY;

    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), ValueCodecDenial> {
        self.0.validate_bounds(bounds)
    }

    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        u64::deserialize_bounded(decoder, bounds).map(|value| Self(value.saturating_add(1)))
    }
}

#[test]
fn encoding_cannot_certify_an_unchecked_decoder_round_trip() {
    let bounds = ValueDecodeBounds::unrestricted();
    let encoded = encode_stored_canonical_value(&Normalizing(15), bounds).unwrap();
    assert_eq!(encoded.raw_text(), Some("15"));
    // The seed reads 15 as 16. Both attempts must compare its re-encoding with
    // the original frozen decimal bytes; the first refusal must acquire no seal.
    for _ in 0..2 {
        assert_eq!(
            decode_stored_canonical_value::<Normalizing>(&encoded, bounds, None),
            Err(ValueCodecDenial::NonCanonical)
        );
    }
}
