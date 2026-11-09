//! The domain text keeps the primitive string codec and its admitted decoder.
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_decl::facade::application_schema::*;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectionGrantKey(String);
impl InspectionGrantKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<String> for InspectionGrantKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for InspectionGrantKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<InspectionGrantKey> for String {
    fn from(value: InspectionGrantKey) -> Self {
        value.0
    }
}
pub struct InspectionGrantKeyBinding;
impl ApplicationScalarValueBinding for InspectionGrantKeyBinding {
    type Value = InspectionGrantKey;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = <StringApplicationValueBinding as ApplicationScalarValueBinding>::Identity;
    type SignedAggregate =
        <StringApplicationValueBinding as ApplicationScalarValueBinding>::SignedAggregate;
    const IDENTITY_NAME: &'static str = "worth.query.fixture.inspection-grant-key.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const SUPPORTS_ADMITTED_DECODE: bool = StringApplicationValueBinding::SUPPORTS_ADMITTED_DECODE;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        StringApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &InspectionGrantKey) -> Result<(), ApplicationValueValidationDenial> {
        StringApplicationValueBinding::validate(&value.0)
    }
    fn encode(value: &InspectionGrantKey) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        StringApplicationValueBinding::encode(&value.0)
    }
}
impl ApplicationReadableScalarValueBinding for InspectionGrantKeyBinding {
    fn decode(value: &AspectValue) -> Result<InspectionGrantKey, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode(value).map(InspectionGrantKey::from)
    }
    fn decode_admitted(
        value: &AspectValue,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<InspectionGrantKey, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode_admitted(value, work).map(InspectionGrantKey::from)
    }
}
impl ApplicationIdentityScalarValueBinding for InspectionGrantKeyBinding {}
