//! The domain text keeps the primitive string codec and its admitted decoder.
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::facade::application_schema::*;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnnotationText(String);
impl AnnotationText {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<String> for AnnotationText {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for AnnotationText {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<AnnotationText> for String {
    fn from(value: AnnotationText) -> Self {
        value.0
    }
}
pub struct AnnotationTextBinding;
impl ApplicationScalarValueBinding for AnnotationTextBinding {
    type Value = AnnotationText;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = <StringApplicationValueBinding as ApplicationScalarValueBinding>::Identity;
    type SignedAggregate =
        <StringApplicationValueBinding as ApplicationScalarValueBinding>::SignedAggregate;
    const IDENTITY_NAME: &'static str = "worth.query.fixture.annotation-text.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const SUPPORTS_ADMITTED_DECODE: bool = StringApplicationValueBinding::SUPPORTS_ADMITTED_DECODE;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        StringApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &AnnotationText) -> Result<(), ApplicationValueValidationDenial> {
        StringApplicationValueBinding::validate(&value.0)
    }
    fn encode(value: &AnnotationText) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        StringApplicationValueBinding::encode(&value.0)
    }
}
impl ApplicationReadableScalarValueBinding for AnnotationTextBinding {
    fn decode(value: &AspectValue) -> Result<AnnotationText, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode(value).map(AnnotationText::from)
    }
    fn decode_admitted(
        value: &AspectValue,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<AnnotationText, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode_admitted(value, work).map(AnnotationText::from)
    }
}
impl ApplicationIdentityScalarValueBinding for AnnotationTextBinding {}
