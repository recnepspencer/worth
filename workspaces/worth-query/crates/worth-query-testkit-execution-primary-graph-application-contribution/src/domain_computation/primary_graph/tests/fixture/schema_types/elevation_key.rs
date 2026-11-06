//! The domain text keeps the primitive string codec and its admitted decoder.
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::facade::application_schema::*;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ElevationKey(String);
impl ElevationKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<String> for ElevationKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for ElevationKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<ElevationKey> for String {
    fn from(value: ElevationKey) -> Self {
        value.0
    }
}
pub struct ElevationKeyBinding;
impl ApplicationScalarValueBinding for ElevationKeyBinding {
    type Value = ElevationKey;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = <StringApplicationValueBinding as ApplicationScalarValueBinding>::Identity;
    type SignedAggregate =
        <StringApplicationValueBinding as ApplicationScalarValueBinding>::SignedAggregate;
    const IDENTITY_NAME: &'static str = "worth.query.fixture.elevation-key.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const SUPPORTS_ADMITTED_DECODE: bool = StringApplicationValueBinding::SUPPORTS_ADMITTED_DECODE;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        StringApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &ElevationKey) -> Result<(), ApplicationValueValidationDenial> {
        StringApplicationValueBinding::validate(&value.0)
    }
    fn encode(value: &ElevationKey) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        StringApplicationValueBinding::encode(&value.0)
    }
}
impl ApplicationReadableScalarValueBinding for ElevationKeyBinding {
    fn decode(value: &AspectValue) -> Result<ElevationKey, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode(value).map(ElevationKey::from)
    }
    fn decode_admitted(
        value: &AspectValue,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<ElevationKey, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode_admitted(value, work).map(ElevationKey::from)
    }
}
impl ApplicationIdentityScalarValueBinding for ElevationKeyBinding {}
