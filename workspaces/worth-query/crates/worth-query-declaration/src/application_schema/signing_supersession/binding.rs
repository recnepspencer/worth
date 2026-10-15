//! Versioned native scalar carrier for append-only supersession facts.
use super::{ApplicationSigningSupersession as Value, ApplicationSigningSupersessionParts};
use crate::application_schema::{
    ApplicationNativeInputDecodeProfile, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
};
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};

pub struct ApplicationSigningSupersessionBinding;
impl ApplicationScalarValueBinding for ApplicationSigningSupersessionBinding {
    type Value = Value;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.query.signing-supersession.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        Some(ApplicationNativeInputDecodeProfile::new(1024, 64, 2048, 8));
    fn validate(value: &Value) -> Result<(), ApplicationValueValidationDenial> {
        let parts = value.parts();
        let mut fields = std::collections::BTreeSet::new();
        if parts.operation.is_empty()
            || parts.changed_dependencies.is_empty()
            || parts.changed_dependencies.iter().any(|field| {
                field.entity().is_empty()
                    || field.aspect().is_empty()
                    || field.field().is_empty()
                    || !fields.insert(field)
            })
        {
            return Err(rejected());
        }
        Ok(())
    }
    fn encode(value: &Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Self::validate(value)?;
        let ceiling = value.parts().maximum_encoded_bytes().ok_or_else(rejected)?;
        let mut bytes = Vec::with_capacity(ceiling);
        serde_json::to_writer(&mut bytes, value.parts()).map_err(|_| rejected())?;
        if bytes.len() > ceiling {
            return Err(rejected().into());
        }
        Ok(AspectValue::String(InternedString::Raw(
            String::from_utf8(bytes).expect("JSON is UTF-8"),
        )))
    }
}
impl ApplicationReadableScalarValueBinding for ApplicationSigningSupersessionBinding {
    fn decode(value: &AspectValue) -> Result<Value, ApplicationValueDecodeDenial> {
        let refused = || ApplicationValueDecodeDenial::CodecRejected {
            binding_identity: Self::IDENTITY,
        };
        let AspectValue::String(InternedString::Raw(text)) = value else {
            return Err(refused());
        };
        let parts: ApplicationSigningSupersessionParts =
            serde_json::from_str(text).map_err(|_| refused())?;
        Value::from_parts(parts).map_err(ApplicationValueDecodeDenial::Validation)
    }
}
fn rejected() -> ApplicationValueValidationDenial {
    ApplicationValueValidationDenial::rejected(
        ApplicationSigningSupersessionBinding::IDENTITY,
        "invalid signing supersession",
    )
}
