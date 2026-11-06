//! The declared value codec of a custom invariant refusal: its wire bound and its decoder,
//! held beside the custom invariant denial that carries the refusal.
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::ScalarAspectType;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_declaration::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationValueDecodeDenial,
};
use worth_relational::facade::runtime::CustomInvariantRefusal;

pub(in crate::domain_computation) fn maximum<Binding: ApplicationScalarValueBinding>(
) -> Result<usize, String> {
    if Binding::SCALAR_FAMILY != ScalarAspectType::String {
        return Err("invariant refusal requires a canonical application value binding".into());
    }
    canonical_wire_maximum::<Binding>()
        .ok_or_else(|| "invariant refusal requires a finite canonical wire bound".into())
}

/// The finite, nonzero wire bound of a canonical String value binding. Every
/// declared domain value (invariant refusal or producer denial) needs one.
pub(crate) fn canonical_wire_maximum<Binding: ApplicationScalarValueBinding>() -> Option<usize> {
    if Binding::SCALAR_FAMILY != ScalarAspectType::String {
        return None;
    }
    Binding::canonical_contract()
        .and_then(|contract| contract.bounds().maximum_wire_bytes())
        .filter(|maximum| *maximum > 0)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantRefusalDecodeDenial {
    NoDomainRefusal,
    TypeMismatch { expected: String, observed: String },
    InvalidEncoding,
    Value(ApplicationValueDecodeDenial),
}

pub(crate) fn decode<Binding: ApplicationReadableScalarValueBinding>(
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
    let maximum = maximum::<Binding>().map_err(|_| Denial::InvalidEncoding)?;
    if bytes.len() > maximum {
        return Err(Denial::InvalidEncoding);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Denial::InvalidEncoding)?;
    Binding::decode(&AspectValue::String(text.to_owned().into())).map_err(Denial::Value)
}
