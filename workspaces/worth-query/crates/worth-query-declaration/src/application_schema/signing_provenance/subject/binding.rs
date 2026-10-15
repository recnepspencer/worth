//! Version-two codec. Old authentication-only bytes are never treated as an empty subject footprint.
use super::{
    ApplicationSigningSubjectProvenance as Value, ApplicationSigningSubjectProvenanceParts,
};
use crate::application_schema::{
    ApplicationNativeInputDecodeProfile, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
};
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};

pub struct ApplicationSigningSubjectProvenanceBinding;
impl ApplicationScalarValueBinding for ApplicationSigningSubjectProvenanceBinding {
    type Value = Value;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.query.signing-subject-provenance.v2";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        Some(ApplicationNativeInputDecodeProfile::new(1024, 64, 2048, 8));
    fn validate(value: &Value) -> Result<(), ApplicationValueValidationDenial> {
        super::super::binding::validate_parts(value.authentication())?;
        let mut captures = std::collections::BTreeSet::new();
        for dependency in value.dependencies() {
            if dependency.value_binding.is_empty()
                || [
                    dependency.source_field.entity(),
                    dependency.source_field.aspect(),
                    dependency.source_field.field(),
                    dependency.captured_field.entity(),
                    dependency.captured_field.aspect(),
                    dependency.captured_field.field(),
                ]
                .iter()
                .any(|text| text.is_empty())
                || !captures.insert(&dependency.captured_field)
            {
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
                reason: "invalid signed-subject provenance",
            }
        })?;
        if bytes.len() > ceiling {
            return Err(rejected().into());
        }
        Ok(AspectValue::String(InternedString::Raw(
            String::from_utf8(bytes).expect("JSON encoder writes UTF-8"),
        )))
    }
}
impl ApplicationReadableScalarValueBinding for ApplicationSigningSubjectProvenanceBinding {
    fn decode(value: &AspectValue) -> Result<Value, ApplicationValueDecodeDenial> {
        let refused = || ApplicationValueDecodeDenial::CodecRejected {
            binding_identity: Self::IDENTITY,
        };
        let AspectValue::String(InternedString::Raw(text)) = value else {
            return Err(refused());
        };
        let parts: ApplicationSigningSubjectProvenanceParts =
            serde_json::from_str(text).map_err(|_| refused())?;
        Value::from_parts(parts).map_err(ApplicationValueDecodeDenial::Validation)
    }
}
fn rejected() -> ApplicationValueValidationDenial {
    ApplicationValueValidationDenial::rejected(
        ApplicationSigningSubjectProvenanceBinding::IDENTITY,
        "invalid signed-subject provenance",
    )
}
