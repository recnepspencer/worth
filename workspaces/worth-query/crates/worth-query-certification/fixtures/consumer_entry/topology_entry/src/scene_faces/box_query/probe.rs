//! Canonical raw probe coordinates interpreted in the selected Scene's local frame.
use super::*;
use worth_query_consumer_values::{Box2Coordinates, IntegerBox2, ModelLocalMetre};

#[derive(Clone, Copy, Debug, Eq, PartialEq, worth_foundational::facade::WorthValue)]
pub struct SceneBoxProbe {
    pub lower_x: u64,
    pub lower_y: u64,
    pub upper_x: u64,
    pub upper_y: u64,
}
impl SceneBoxProbe {
    pub const fn raw(self) -> Box2Coordinates<u64> {
        Box2Coordinates {
            lower: [self.lower_x, self.lower_y],
            upper: [self.upper_x, self.upper_y],
        }
    }
}
pub struct SceneBoxProbeBinding;
const BOUNDS: ValueDecodeBounds = ValueDecodeBounds::new(1024, 1024, 16, 8);
impl ApplicationScalarValueBinding for SceneBoxProbeBinding {
    type Value = SceneBoxProbe;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.query.scene-face.aabb-metre-model-local.v1";
    const SCALAR_FAMILY: worth_query_decl::facade::typed::ScalarAspectType =
        worth_query_decl::facade::typed::ScalarAspectType::String;
    const UNIT: Option<ApplicationUnitIdentity> = Some(ApplicationUnitIdentity::declared(
        IntegerBox2::<ModelLocalMetre>::LENGTH_UNIT,
    ));
    const FRAME: Option<ApplicationFrameIdentity> = Some(ApplicationFrameIdentity::declared(
        IntegerBox2::<ModelLocalMetre>::FRAME,
    ));
    const SUPPORTS_ADMITTED_DECODE: bool = <SceneBoxProbe as CanonicalValue>::SUPPORTS_ADMISSION;
    fn canonical_contract() -> Option<ApplicationCanonicalValueContract> {
        Some(ApplicationCanonicalValueContract::of::<SceneBoxProbe>(
            BOUNDS,
        ))
    }
    fn validate(_: &SceneBoxProbe) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(value: &SceneBoxProbe) -> Result<ApplicationValue, ApplicationValueEncodeDenial> {
        encode_canonical_value(value, BOUNDS)
            .map(|text| ApplicationValue::String(text.into()))
            .map_err(|cause| ApplicationValueEncodeDenial::Canonical {
                binding_identity: Self::IDENTITY,
                cause,
            })
    }
}
impl ApplicationReadableScalarValueBinding for SceneBoxProbeBinding {
    fn decode(value: &ApplicationValue) -> Result<SceneBoxProbe, ApplicationValueDecodeDenial> {
        decode_application_canonical_value(value, BOUNDS, Self::IDENTITY)
    }
    fn decode_admitted(
        value: &ApplicationValue,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<SceneBoxProbe, ApplicationValueDecodeDenial> {
        decode_application_canonical_value_with_admission(value, BOUNDS, Self::IDENTITY, admission)
    }
}
