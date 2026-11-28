//! Typed decoding of declared domain refusal payloads on custom invariant denials.

use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    ApplicationValueDecodeDenial,
};
use worth_relational::facade::runtime::CustomInvariantRefusal;

use super::InvariantViolationFields;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantRefusalDecodeDenial {
    NoDomainRefusal,
    TypeMismatch { expected: String, observed: String },
    InvalidEncoding,
    Value(ApplicationValueDecodeDenial),
}

/// The finite, nonzero wire bound of a canonical String value binding.
pub fn canonical_wire_maximum<Binding: ApplicationScalarValueBinding>() -> Option<usize> {
    if Binding::SCALAR_FAMILY != ScalarAspectType::String {
        return None;
    }
    Binding::canonical_contract()
        .and_then(|contract| contract.bounds().maximum_wire_bytes())
        .filter(|maximum| *maximum > 0)
}

pub fn decode_refusal_payload<Binding: ApplicationReadableScalarValueBinding>(
    refusal: &CustomInvariantRefusal,
) -> Result<Binding::Value, WorthQueryInvariantRefusalDecodeDenial> {
    use WorthQueryInvariantRefusalDecodeDenial as Denial;
    let CustomInvariantRefusal::DeclaredValue {
        value_identity,
        bytes,
    } = refusal
    else {
        return Err(Denial::NoDomainRefusal);
    };
    if value_identity != Binding::IDENTITY_NAME {
        return Err(Denial::TypeMismatch {
            expected: Binding::IDENTITY_NAME.to_owned(),
            observed: value_identity.clone(),
        });
    }
    let maximum = canonical_wire_maximum::<Binding>().ok_or(Denial::InvalidEncoding)?;
    if bytes.len() > maximum {
        return Err(Denial::InvalidEncoding);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Denial::InvalidEncoding)?;
    Binding::decode(&AspectValue::String(text.to_owned().into())).map_err(Denial::Value)
}

/// Decode an application refusal carried by the original Relational invariant fields.
pub fn decode_invariant_fields<Binding: ApplicationReadableScalarValueBinding>(
    fields: &InvariantViolationFields,
) -> Result<Binding::Value, WorthQueryInvariantRefusalDecodeDenial> {
    let refusal = invariant_refusal_payload(fields)
        .ok_or(WorthQueryInvariantRefusalDecodeDenial::NoDomainRefusal)?;
    decode_refusal_payload::<Binding>(refusal)
}

pub fn invariant_refusal_payload(
    fields: &InvariantViolationFields,
) -> Option<&CustomInvariantRefusal> {
    match fields {
        InvariantViolationFields::CustomInvariantViolation { refusal, .. } => Some(refusal),
        InvariantViolationFields::CustomInvariantFailure { cause, .. } => match cause.as_deref()? {
            worth_relational::facade::runtime::CustomInvariantFailureCause::DeclaredValue(
                refusal,
            ) => Some(refusal),
            _ => None,
        },
        _ => None,
    }
}
