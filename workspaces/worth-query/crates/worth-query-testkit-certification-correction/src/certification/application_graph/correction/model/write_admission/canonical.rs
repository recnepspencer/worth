//! JSON domain owners whose construction seeds reject or normalize encoder output.
use worth_foundational::facade::{
    AspectValue, InternedString, ScalarAspectType, ValueSeedDelegate, ValueShapeBody,
};
use worth_query_decl::facade::application_schema::{
    decode_delegated_canonical_value_with_admission, encode_delegated_canonical_value,
    ApplicationCanonicalValueContract, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial, CanonicalValue,
    NoApplicationUnit, ValueCodecDenial, ValueDecodeAdmission, ValueDecodeBounds,
};
pub const BOUNDS: ValueDecodeBounds = ValueDecodeBounds::new(20, 20, 1, 1);

pub struct CounterSeed;
impl ValueSeedDelegate for CounterSeed {
    type Value = u64;
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = <u64 as CanonicalValue>::SHAPE_BODY;
    fn validate(value: &u64, bounds: ValueDecodeBounds) -> Result<(), ValueCodecDenial> {
        value.validate_bounds(bounds)
    }
    fn deserialize<'de, D: serde::Deserializer<'de>>(
        decoder: D,
        bounds: ValueDecodeBounds,
        admission: Option<&mut ValueDecodeAdmission<'_>>,
    ) -> Result<u64, D::Error> {
        let value = match admission {
            Some(admission) => u64::deserialize_admitted(decoder, bounds, admission)?,
            None => u64::deserialize_bounded(decoder, bounds)?,
        };
        match value {
            13 => Err(serde::de::Error::custom(
                "owner refuses this authored spelling",
            )),
            15 => Ok(14), // A corrupt constructor must also fail canonical comparison.
            value => Ok(value),
        }
    }
}
#[derive(Clone, Debug, serde::Serialize)]
#[serde(transparent)]
pub struct CanonicalCounter(pub u64);
impl CanonicalValue for CanonicalCounter {
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = <CounterSeed as ValueSeedDelegate>::SHAPE_BODY;
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), ValueCodecDenial> {
        CounterSeed::validate(&self.0, bounds)
    }
    fn deserialize_bounded<'de, D: serde::Deserializer<'de>>(
        decoder: D,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, D::Error> {
        CounterSeed::deserialize(decoder, bounds, None).map(Self)
    }
    fn deserialize_admitted<'de, D: serde::Deserializer<'de>>(
        decoder: D,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, D::Error> {
        CounterSeed::deserialize(decoder, bounds, Some(admission)).map(Self)
    }
}
worth_query_decl::facade::worth_query_value_binding!(pub CanonicalBinding for CanonicalCounter {
    identity: "domain-write.canonical.v1", scalar: String, canonical: BOUNDS, domain_value,
});

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DelegatedCounter(pub u64);
pub struct DelegatedBinding;
impl ApplicationScalarValueBinding for DelegatedBinding {
    type Value = DelegatedCounter;
    type Unit = NoApplicationUnit;
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "domain-write.delegated.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const DOMAIN_VALUE: bool = true;
    const SUPPORTS_ADMITTED_DECODE: bool = true;
    fn canonical_contract() -> Option<ApplicationCanonicalValueContract> {
        Some(ApplicationCanonicalValueContract::domain_delegated::<
            CounterSeed,
        >(BOUNDS))
    }
    fn validate(_: &DelegatedCounter) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(value: &DelegatedCounter) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        encode_delegated_canonical_value::<CounterSeed>(&value.0, BOUNDS)
            .map(|value| AspectValue::String(InternedString::Raw(value)))
            .map_err(|cause| ApplicationValueEncodeDenial::Canonical {
                binding_identity: Self::IDENTITY,
                cause,
            })
    }
}
impl ApplicationReadableScalarValueBinding for DelegatedBinding {
    fn decode(value: &AspectValue) -> Result<DelegatedCounter, ApplicationValueDecodeDenial> {
        Self::decode_admitted(value, &mut |_, _| Ok(()))
    }
    fn decode_admitted(
        value: &AspectValue,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<DelegatedCounter, ApplicationValueDecodeDenial> {
        let Some(raw) = value.as_string() else {
            return Err(ApplicationValueDecodeDenial::CodecRejected {
                binding_identity: Self::IDENTITY,
            });
        };
        decode_delegated_canonical_value_with_admission::<CounterSeed>(raw, BOUNDS, admission)
            .map(DelegatedCounter)
            .map_err(|cause| ApplicationValueDecodeDenial::Canonical {
                binding_identity: Self::IDENTITY,
                cause,
            })
    }
}
