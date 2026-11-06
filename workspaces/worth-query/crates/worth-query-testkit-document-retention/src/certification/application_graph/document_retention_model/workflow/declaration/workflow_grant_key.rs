//! The domain text keeps the primitive string codec and its admitted decoder.
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_decl::facade::application_schema::*;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowGrantKey(String);
impl WorkflowGrantKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<String> for WorkflowGrantKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for WorkflowGrantKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<WorkflowGrantKey> for String {
    fn from(value: WorkflowGrantKey) -> Self {
        value.0
    }
}
pub struct WorkflowGrantKeyBinding;
impl ApplicationScalarValueBinding for WorkflowGrantKeyBinding {
    type Value = WorkflowGrantKey;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = <StringApplicationValueBinding as ApplicationScalarValueBinding>::Identity;
    type SignedAggregate =
        <StringApplicationValueBinding as ApplicationScalarValueBinding>::SignedAggregate;
    const IDENTITY_NAME: &'static str = "worth.query.fixture.workflow-grant-key.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const SUPPORTS_ADMITTED_DECODE: bool = StringApplicationValueBinding::SUPPORTS_ADMITTED_DECODE;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        StringApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &WorkflowGrantKey) -> Result<(), ApplicationValueValidationDenial> {
        StringApplicationValueBinding::validate(&value.0)
    }
    fn encode(value: &WorkflowGrantKey) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        StringApplicationValueBinding::encode(&value.0)
    }
}
impl ApplicationReadableScalarValueBinding for WorkflowGrantKeyBinding {
    fn decode(value: &AspectValue) -> Result<WorkflowGrantKey, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode(value).map(WorkflowGrantKey::from)
    }
    fn decode_admitted(
        value: &AspectValue,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<WorkflowGrantKey, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode_admitted(value, work).map(WorkflowGrantKey::from)
    }
}
impl ApplicationIdentityScalarValueBinding for WorkflowGrantKeyBinding {}
