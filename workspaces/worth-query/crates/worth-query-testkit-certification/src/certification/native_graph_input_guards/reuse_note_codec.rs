//! Installed scalar admission rejects malformed content before any reuse decision.
pub use worth_foundational::facade::{AspectValue, ScalarAspectType};
pub use worth_query_decl::facade::application_schema::*;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Note(pub String);
pub struct NoteCodec;
impl ApplicationScalarValueBinding for NoteCodec {
    type Value = Note;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.query.reuse.note-codec.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        StringApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &Note) -> Result<(), ApplicationValueValidationDenial> {
        if value.0 == "malformed" {
            return Err(ApplicationValueValidationDenial::rejected(
                Self::IDENTITY,
                "malformed note",
            ));
        }
        StringApplicationValueBinding::validate(&value.0)
    }
    fn encode(value: &Note) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Self::validate(value)?;
        StringApplicationValueBinding::encode(&value.0)
    }
}
impl ApplicationReadableScalarValueBinding for NoteCodec {
    fn decode(value: &AspectValue) -> Result<Note, ApplicationValueDecodeDenial> {
        let decoded = StringApplicationValueBinding::decode(value)?;
        let note = Note(decoded);
        Self::validate(&note)?;
        Ok(note)
    }
}
