//! The domain text keeps the primitive string codec and its admitted decoder.
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::facade::application_schema::*;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantKey(String);
impl GrantKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<String> for GrantKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for GrantKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<GrantKey> for String {
    fn from(value: GrantKey) -> Self {
        value.0
    }
}
pub struct GrantKeyBinding;
impl ApplicationScalarValueBinding for GrantKeyBinding {
    type Value = GrantKey;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = <StringApplicationValueBinding as ApplicationScalarValueBinding>::Identity;
    type SignedAggregate =
        <StringApplicationValueBinding as ApplicationScalarValueBinding>::SignedAggregate;
    const IDENTITY_NAME: &'static str = "worth.query.fixture.grant-key.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const SUPPORTS_ADMITTED_DECODE: bool = StringApplicationValueBinding::SUPPORTS_ADMITTED_DECODE;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        StringApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &GrantKey) -> Result<(), ApplicationValueValidationDenial> {
        StringApplicationValueBinding::validate(&value.0)
    }
    fn encode(value: &GrantKey) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        StringApplicationValueBinding::encode(&value.0)
    }
}
impl ApplicationReadableScalarValueBinding for GrantKeyBinding {
    fn decode(value: &AspectValue) -> Result<GrantKey, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode(value).map(GrantKey::from)
    }
    fn decode_admitted(
        value: &AspectValue,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<GrantKey, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode_admitted(value, work).map(GrantKey::from)
    }
}
impl ApplicationIdentityScalarValueBinding for GrantKeyBinding {}
