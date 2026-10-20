//! A bounded typed scalar codec, not a serialized authentication authority token.
use super::{ApplicationSigningProvenance as Value, ApplicationSigningProvenanceParts};
use crate::application_schema::{
    ApplicationNativeInputDecodeProfile, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
};
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};

pub struct ApplicationSigningProvenanceBinding;
impl ApplicationScalarValueBinding for ApplicationSigningProvenanceBinding {
    type Value = Value;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.query.signing-provenance.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        Some(ApplicationNativeInputDecodeProfile::new(512, 32, 1024, 6));

    fn validate(value: &Value) -> Result<(), ApplicationValueValidationDenial> {
        validate_parts(value.parts())
    }
    fn encode(value: &Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Self::validate(value)?;
        let ceiling = value
            .parts()
            .metadata
            .maximum_encoded_bytes()
            .ok_or_else(rejected)?;
        let mut bytes = Vec::with_capacity(ceiling);
        serde_json::to_writer(&mut bytes, value.parts()).map_err(|_| {
            ApplicationValueEncodeDenial::CodecRejected {
                binding_identity: Self::IDENTITY,
                reason: "invalid signing provenance",
            }
        })?;
        if bytes.len() > ceiling {
            return Err(rejected().into());
        }
        let text = String::from_utf8(bytes).expect("JSON encoder writes UTF-8");
        Ok(AspectValue::String(InternedString::Raw(text)))
    }
}
impl ApplicationReadableScalarValueBinding for ApplicationSigningProvenanceBinding {
    fn decode(value: &AspectValue) -> Result<Value, ApplicationValueDecodeDenial> {
        let refused = || ApplicationValueDecodeDenial::CodecRejected {
            binding_identity: Self::IDENTITY,
        };
        let AspectValue::String(InternedString::Raw(text)) = value else {
            return Err(refused());
        };
        let parts: ApplicationSigningProvenanceParts =
            serde_json::from_str(text).map_err(|_| refused())?;
        Value::from_parts(parts).map_err(ApplicationValueDecodeDenial::Validation)
    }
}
fn rejected() -> ApplicationValueValidationDenial {
    ApplicationValueValidationDenial::rejected(
        ApplicationSigningProvenanceBinding::IDENTITY,
        "invalid signing provenance",
    )
}

pub(super) fn validate_parts(
    parts: &ApplicationSigningProvenanceParts,
) -> Result<(), ApplicationValueValidationDenial> {
    let metadata = &parts.metadata;
    for text in [
        metadata.program_revision.as_str(),
        metadata.operation.as_str(),
        metadata.binding.as_str(),
        metadata.clock_source.as_str(),
        metadata.clock_timeline.as_str(),
    ] {
        if text.is_empty() {
            return Err(rejected());
        }
    }
    if crate::authentication::WorthQueryAuthenticationPurpose::validate(&metadata.purpose).is_err()
        || crate::authentication::WorthQueryExternalPrincipalIdentity::validate_components(
            &metadata.signer_issuer,
            &metadata.signer_subject,
        )
        .is_err()
        || crate::authentication::WorthQueryAuthenticationVerifierIdentity::validate(
            &metadata.verification_configuration,
        )
        .is_err()
        || parts.observed_at_nanoseconds < parts.issued_at_nanoseconds
    {
        return Err(rejected());
    }
    Ok(())
}
