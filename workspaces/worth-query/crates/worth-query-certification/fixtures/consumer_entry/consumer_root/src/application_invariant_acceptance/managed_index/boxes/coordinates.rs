//! A finite canonical representation of the numerical owner's explicit unit/frame.
use worth_query_consumer_values::{Box2, Box2Coordinates};
use worth_query_decl::facade::application_schema::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, worth_foundational::facade::WorthValue)]
pub struct Coordinates {
    pub lower_x: i64,
    pub lower_y: i64,
    pub upper_x: i64,
    pub upper_y: i64,
}
impl Coordinates {
    pub const fn new(lower_x: i64, lower_y: i64, upper_x: i64, upper_y: i64) -> Self {
        Self {
            lower_x,
            lower_y,
            upper_x,
            upper_y,
        }
    }
    pub const fn raw(self) -> Box2Coordinates {
        Box2Coordinates {
            lower: [self.lower_x, self.lower_y],
            upper: [self.upper_x, self.upper_y],
        }
    }
}
pub struct Binding;
const BOUNDS: ValueDecodeBounds = ValueDecodeBounds::new(1024, 1024, 16, 8);
impl ApplicationScalarValueBinding for Binding {
    type Value = Coordinates;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "managed-catalog.box2.integer-xy.v1";
    const SCALAR_FAMILY: worth_query_decl::facade::typed::ScalarAspectType =
        worth_query_decl::facade::typed::ScalarAspectType::String;
    const UNIT: Option<ApplicationUnitIdentity> =
        Some(ApplicationUnitIdentity::declared(Box2::LENGTH_UNIT));
    const FRAME: Option<ApplicationFrameIdentity> =
        Some(ApplicationFrameIdentity::declared(Box2::FRAME));
    const SUPPORTS_ADMITTED_DECODE: bool = <Coordinates as CanonicalValue>::SUPPORTS_ADMISSION;
    fn canonical_contract() -> Option<ApplicationCanonicalValueContract> {
        Some(ApplicationCanonicalValueContract::of::<Coordinates>(BOUNDS))
    }
    fn validate(_: &Coordinates) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(value: &Coordinates) -> Result<ApplicationValue, ApplicationValueEncodeDenial> {
        encode_canonical_value(value, BOUNDS)
            .map(|text| ApplicationValue::String(text.into()))
            .map_err(|cause| ApplicationValueEncodeDenial::Canonical {
                binding_identity: Self::IDENTITY,
                cause,
            })
    }
}
impl ApplicationReadableScalarValueBinding for Binding {
    fn decode(value: &ApplicationValue) -> Result<Coordinates, ApplicationValueDecodeDenial> {
        decode_application_canonical_value::<Coordinates>(value, BOUNDS, Self::IDENTITY)
    }
    fn decode_admitted(
        value: &ApplicationValue,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Coordinates, ApplicationValueDecodeDenial> {
        decode_application_canonical_value_with_admission::<Coordinates>(
            value,
            BOUNDS,
            Self::IDENTITY,
            admission,
        )
    }
}
