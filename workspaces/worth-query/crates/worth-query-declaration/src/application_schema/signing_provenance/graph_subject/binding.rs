//! V3 is a distinct codec; v1/v2 bytes and bindings retain their original meaning.
use super::{
    ApplicationSigningGraphSubjectProvenance as Value,
    ApplicationSigningGraphSubjectProvenanceParts,
};
use crate::application_schema::{
    ApplicationNativeInputDecodeProfile, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
};
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};
pub struct ApplicationSigningGraphSubjectProvenanceBinding;
impl ApplicationScalarValueBinding for ApplicationSigningGraphSubjectProvenanceBinding {
    type Value = Value;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.query.signing-graph-subject-provenance.v3";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        Some(ApplicationNativeInputDecodeProfile::new(1024, 64, 4096, 8));
    fn validate(value: &Value) -> Result<(), ApplicationValueValidationDenial> {
        super::super::binding::validate_parts(value.authentication())?;
        super::super::subject::validate_dependencies(
            value
                .dependencies()
                .iter()
                .map(|d| (&d.source_field, &d.captured_field, d.value_binding.as_str())),
        )
        .map_err(|()| rejected())?;
        if value.selections().iter().any(|selection| {
            selection.relation.is_empty()
                || !value
                    .dependencies()
                    .iter()
                    .any(|dependency| dependency.captured_field == selection.captured_field)
        }) {
            return Err(rejected());
        }
        for dependency in value.dependencies() {
            let mut cursor = value.authentication().subject;
            for selection in value
                .selections()
                .iter()
                .filter(|selection| selection.captured_field == dependency.captured_field)
            {
                if selection.anchor != cursor {
                    return Err(rejected());
                }
                cursor = selection.selected;
            }
            if cursor != dependency.source {
                return Err(rejected());
            }
        }
        Ok(())
    }
    fn encode(value: &Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Self::validate(value)?;
        let ceiling = value.parts().maximum_encoded_bytes().ok_or_else(rejected)?;
        let mut bytes = Vec::with_capacity(ceiling);
        serde_json::to_writer(&mut bytes, value.parts()).map_err(|_| {
            ApplicationValueEncodeDenial::CodecRejected {
                binding_identity: Self::IDENTITY,
                reason: "invalid graph signed-subject provenance",
            }
        })?;
        if bytes.len() > ceiling {
            return Err(rejected().into());
        }
        Ok(AspectValue::String(InternedString::Raw(
            String::from_utf8(bytes).expect("JSON writes UTF-8"),
        )))
    }
}
impl ApplicationReadableScalarValueBinding for ApplicationSigningGraphSubjectProvenanceBinding {
    fn decode(value: &AspectValue) -> Result<Value, ApplicationValueDecodeDenial> {
        let refused = || ApplicationValueDecodeDenial::CodecRejected {
            binding_identity: Self::IDENTITY,
        };
        let AspectValue::String(InternedString::Raw(text)) = value else {
            return Err(refused());
        };
        let parts: ApplicationSigningGraphSubjectProvenanceParts =
            serde_json::from_str(text).map_err(|_| refused())?;
        Value::from_parts(parts).map_err(ApplicationValueDecodeDenial::Validation)
    }
}
fn rejected() -> ApplicationValueValidationDenial {
    ApplicationValueValidationDenial::rejected(
        ApplicationSigningGraphSubjectProvenanceBinding::IDENTITY,
        "invalid graph signed-subject provenance",
    )
}
