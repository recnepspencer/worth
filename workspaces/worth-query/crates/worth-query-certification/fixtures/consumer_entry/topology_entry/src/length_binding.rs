//! The existing fixed UInt64 length wire with explicit original admitted decoding.
use super::fixture_values::PositiveLength;
use worth_query_decl::facade::application_schema::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TopologyLengthBinding;
impl ApplicationScalarValueBinding for TopologyLengthBinding {
    type Value = PositiveLength;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.query.certification.positive-length.v1";
    const SCALAR_FAMILY: worth_query_decl::facade::typed::ScalarAspectType =
        worth_query_decl::facade::typed::ScalarAspectType::UInt64;
    const UNIT: Option<ApplicationUnitIdentity> = Some(ApplicationUnitIdentity::declared("Metre"));
    const FRAME: Option<ApplicationFrameIdentity> = Some(ApplicationFrameIdentity::declared(
        "worth.frames.model-local.v1",
    ));
    const SUPPORTS_ADMITTED_DECODE: bool = true;
    fn validate(_: &PositiveLength) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(value: &PositiveLength) -> Result<ApplicationValue, ApplicationValueEncodeDenial> {
        Ok(ApplicationValue::UInt64(PositiveLength::get(value)))
    }
}
impl ApplicationReadableScalarValueBinding for TopologyLengthBinding {
    fn decode(value: &ApplicationValue) -> Result<PositiveLength, ApplicationValueDecodeDenial> {
        let ApplicationValue::UInt64(value) = value else {
            return Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
                binding_identity: Self::IDENTITY,
                expected: Self::SCALAR_FAMILY,
                observed: value.value_family(),
            });
        };
        PositiveLength::new(*value).ok_or(ApplicationValueDecodeDenial::CodecRejected {
            binding_identity: Self::IDENTITY,
        })
    }
    fn decode_admitted(
        value: &ApplicationValue,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<PositiveLength, ApplicationValueDecodeDenial> {
        if value.value_family() != Self::SCALAR_FAMILY {
            return Self::decode(value);
        }
        // The carrier and domain constructor are fixed work and allocate nothing.
        admission
            .admit(1, 0)
            .map_err(|cause| ApplicationValueDecodeDenial::Canonical {
                binding_identity: Self::IDENTITY,
                cause,
            })?;
        Self::decode(value)
    }
}
