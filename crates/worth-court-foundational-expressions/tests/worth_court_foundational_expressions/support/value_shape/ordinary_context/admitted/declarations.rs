// Representation bounds do not replace cumulative borrowed request authority.
pub(crate) use {
    crate::value_shape::ordinary_context::packet, crate::value_shape::ordinary_context::Packet,
    crate::value_shape::ordinary_context::BOUNDS, crate::value_shape::ordinary_context::RAW,
};
pub(crate) use {
    worth_foundational::facade::decode_canonical_value_with_admission,
    worth_foundational::facade::CanonicalValue, worth_foundational::facade::ValueCodecDenial as D,
    worth_foundational::facade::ValueDecodeAdmission,
    worth_foundational::facade::ValueDecodeBounds, worth_foundational::facade::ValueShapeBody,
    worth_foundational::facade::WorthValue,
};

#[derive(Debug, PartialEq, serde::Serialize)]
pub(crate) struct Swallowing(pub(crate) bool);
impl CanonicalValue for Swallowing {
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = { bool::SHAPE_BODY };
    fn validate_bounds(&self, _: ValueDecodeBounds) -> Result<(), D> {
        Ok(())
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        bool::deserialize_bounded(decoder, bounds).map(Self)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        let _ignored = admission.admit(1, 1000);
        Self::deserialize_bounded(decoder, bounds)
    }
}

#[derive(Debug, WorthValue)]
pub(crate) struct LongGrammar {
    #[serde(
        rename = "a_static_declared_field_name_whose_size_is_unrelated_to_the_two_byte_empty_object_and_must_never_be_rendered_as_an_unfunded_diagnostic_in_a_native_request"
    )]
    pub(crate) value: bool,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub(crate) struct Unadmitted(pub(crate) bool);
impl CanonicalValue for Unadmitted {
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = { bool::SHAPE_BODY };
    fn validate_bounds(&self, _: ValueDecodeBounds) -> Result<(), D> {
        Ok(())
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        bool::deserialize_bounded(decoder, bounds).map(Self)
    }
}
