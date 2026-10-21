//! Installed scalar admission rejects malformed content before any reuse decision.
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_decl::facade::application_schema::*;
pub struct NoteCodec;
impl ApplicationScalarValueBinding for NoteCodec {
    type Value = String;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.query.reuse.note-codec.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        StringApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &String) -> Result<(), ApplicationValueValidationDenial> {
        if value == "malformed" {
            return Err(ApplicationValueValidationDenial::rejected(
                Self::IDENTITY,
                "malformed note",
            ));
        }
        StringApplicationValueBinding::validate(value)
    }
    fn encode(value: &String) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Self::validate(value)?;
        StringApplicationValueBinding::encode(value)
    }
}
impl ApplicationReadableScalarValueBinding for NoteCodec {
    fn decode(value: &AspectValue) -> Result<String, ApplicationValueDecodeDenial> {
        let decoded = StringApplicationValueBinding::decode(value)?;
        Self::validate(&decoded)?;
        Ok(decoded)
    }
}
