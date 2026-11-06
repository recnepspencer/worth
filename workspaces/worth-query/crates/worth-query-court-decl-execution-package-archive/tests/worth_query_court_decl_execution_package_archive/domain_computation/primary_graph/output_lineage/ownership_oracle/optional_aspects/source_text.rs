//! The domain text keeps the primitive string codec and its admitted decoder.
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::facade::application_schema::*;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceText(String);
impl SourceText {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<String> for SourceText {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for SourceText {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<SourceText> for String {
    fn from(value: SourceText) -> Self {
        value.0
    }
}
pub struct SourceTextBinding;
impl ApplicationScalarValueBinding for SourceTextBinding {
    type Value = SourceText;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = <StringApplicationValueBinding as ApplicationScalarValueBinding>::Identity;
    type SignedAggregate =
        <StringApplicationValueBinding as ApplicationScalarValueBinding>::SignedAggregate;
    const IDENTITY_NAME: &'static str = "worth.query.fixture.source-text.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const SUPPORTS_ADMITTED_DECODE: bool = StringApplicationValueBinding::SUPPORTS_ADMITTED_DECODE;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        StringApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &SourceText) -> Result<(), ApplicationValueValidationDenial> {
        StringApplicationValueBinding::validate(&value.0)
    }
    fn encode(value: &SourceText) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        StringApplicationValueBinding::encode(&value.0)
    }
}
impl ApplicationReadableScalarValueBinding for SourceTextBinding {
    fn decode(value: &AspectValue) -> Result<SourceText, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode(value).map(SourceText::from)
    }
    fn decode_admitted(
        value: &AspectValue,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<SourceText, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode_admitted(value, work).map(SourceText::from)
    }
}
impl ApplicationIdentityScalarValueBinding for SourceTextBinding {}
