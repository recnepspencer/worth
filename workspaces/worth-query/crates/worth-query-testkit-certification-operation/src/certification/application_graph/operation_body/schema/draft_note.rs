//! The domain text keeps the primitive string codec and its admitted decoder.
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_decl::facade::application_schema::*;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftNote(String);
impl DraftNote {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<String> for DraftNote {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for DraftNote {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<DraftNote> for String {
    fn from(value: DraftNote) -> Self {
        value.0
    }
}
pub struct DraftNoteBinding;
impl ApplicationScalarValueBinding for DraftNoteBinding {
    type Value = DraftNote;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = <StringApplicationValueBinding as ApplicationScalarValueBinding>::Identity;
    type SignedAggregate =
        <StringApplicationValueBinding as ApplicationScalarValueBinding>::SignedAggregate;
    const IDENTITY_NAME: &'static str = "worth.query.fixture.draft-note.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const SUPPORTS_ADMITTED_DECODE: bool = StringApplicationValueBinding::SUPPORTS_ADMITTED_DECODE;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        StringApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &DraftNote) -> Result<(), ApplicationValueValidationDenial> {
        StringApplicationValueBinding::validate(&value.0)
    }
    fn encode(value: &DraftNote) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        StringApplicationValueBinding::encode(&value.0)
    }
}
impl ApplicationReadableScalarValueBinding for DraftNoteBinding {
    fn decode(value: &AspectValue) -> Result<DraftNote, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode(value).map(DraftNote::from)
    }
    fn decode_admitted(
        value: &AspectValue,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<DraftNote, ApplicationValueDecodeDenial> {
        StringApplicationValueBinding::decode_admitted(value, work).map(DraftNote::from)
    }
}
impl ApplicationIdentityScalarValueBinding for DraftNoteBinding {}
