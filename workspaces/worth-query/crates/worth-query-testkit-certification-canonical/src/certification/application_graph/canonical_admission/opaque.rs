//! An opaque owner deliberately swallowing the real admission callback's refusal.
pub use super::model::NestedValue;
pub use worth_foundational::facade::{
    CanonicalValue, ValueCodecDenial, ValueDecodeAdmission, ValueDecodeBounds, ValueShapeBody,
};
thread_local! { static IGNORED_REFUSALS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }; }
thread_local! {
    static CONSULT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DECODES: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static SECOND_DECODE: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}
pub(crate) fn begin_request() {
    CONSULT.set(0);
    DECODES.set(0);
    SECOND_DECODE.set(None);
}
pub(crate) fn consult(ordinal: usize) {
    CONSULT.set(ordinal);
}
pub(crate) fn second_decode_consult() -> Option<usize> {
    SECOND_DECODE.get()
}

pub fn reset() {
    IGNORED_REFUSALS.set(0);
}
pub fn ignored_refusals() -> u32 {
    IGNORED_REFUSALS.get()
}

#[derive(Clone, Debug, PartialEq)]
pub struct SwallowingValue(pub NestedValue);
impl serde::Serialize for SwallowingValue {
    fn serialize<Serializer: serde::Serializer>(
        &self,
        serializer: Serializer,
    ) -> Result<Serializer::Ok, Serializer::Error> {
        self.0.serialize(serializer)
    }
}
impl CanonicalValue for SwallowingValue {
    const WIRE_NULLABLE: bool = NestedValue::WIRE_NULLABLE;
    const SUPPORTS_ADMISSION: bool = true;
    const SHAPE_BODY: ValueShapeBody = { NestedValue::SHAPE_BODY };
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), ValueCodecDenial> {
        self.0.validate_bounds(bounds)
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        NestedValue::deserialize_bounded(decoder, bounds).map(Self)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        // Every cost originates in the real supported nested decoder. No made-up
        // stop, callback error, or budget arithmetic supplies the negative case.
        DECODES.set(DECODES.get() + 1);
        let second = DECODES.get() == 2;
        NestedValue::deserialize_admitted(decoder, bounds, &mut |work, bytes| {
            let result = admission.admit(work, bytes);
            if second && work > 0 && SECOND_DECODE.get().is_none() {
                SECOND_DECODE.set(Some(CONSULT.get()));
            }
            if result.is_err() {
                IGNORED_REFUSALS.set(IGNORED_REFUSALS.get() + 1);
            }
            Ok(())
        })
        .map(Self)
    }
}
