use super::*;

pub struct BoxDelegate<Child>(PhantomData<fn() -> Child>);
impl<Child: ValueSeedDelegate> ValueSeedDelegate for BoxDelegate<Child> {
    type Value = Box<Child::Value>;
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = Child::WIRE_NULLABLE;
    const SHAPE_BODY: ValueShapeBody = { ValueShapeBody::Transparent(Child::EDGE) };
    fn validate(value: &Self::Value, bounds: ValueDecodeBounds) -> Result<(), D> {
        Child::validate(value, bounds)
    }
    fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        mut admission: Option<&mut ValueDecodeAdmission<'_>>,
    ) -> Result<Self::Value, Decoder::Error> {
        if let Some(admit) = admission.as_deref_mut() {
            admit(1, std::mem::size_of::<Child::Value>()).map_err(serde::de::Error::custom)?;
        }
        DelegatedValueSeed::<Child>::new(bounds, admission)
            .deserialize(decoder)
            .map(Box::new)
    }
}

impl<Value: CanonicalValue> CanonicalValue for Box<Value> {
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = Value::WIRE_NULLABLE;
    const SHAPE_BODY: ValueShapeBody = { BoxDelegate::<ValueDelegate<Value>>::SHAPE_BODY };
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), D> {
        BoxDelegate::<ValueDelegate<Value>>::validate(self, bounds)
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        BoxDelegate::<ValueDelegate<Value>>::deserialize(decoder, bounds, None)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        BoxDelegate::<ValueDelegate<Value>>::deserialize(decoder, bounds, Some(admission))
    }
}
