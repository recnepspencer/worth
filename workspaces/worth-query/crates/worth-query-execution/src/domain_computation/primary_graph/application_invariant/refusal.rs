//! The declared value codec is the sole refusal encoder and decoder.

use crate::domain_computation::primary_graph::vocabulary::application_invariant::refusal::maximum;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    ApplicationValueDecodeDenial, ValueDecodeBounds,
};
use worth_relational::facade::runtime::{
    CustomInvariantExecutionError, CustomInvariantLeaseBudget, CustomInvariantRefusal,
    CustomInvariantVerdict,
};

worth_query_declaration::worth_query_value_binding!(pub WorthQueryEmptyInvariantRefusal for () {
    identity: "worth.query.empty-invariant-refusal.v1", scalar: String,
    canonical: ValueDecodeBounds::new(4, 1, 1, 1),
});

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantRefusalDecodeDenial {
    NoDomainRefusal,
    TypeMismatch { expected: String, observed: String },
    InvalidEncoding,
    Value(ApplicationValueDecodeDenial),
}

pub(super) fn encode<Binding: ApplicationReadableScalarValueBinding>(
    verdict: super::WorthQueryApplicationInvariantVerdict<Binding::Value>,
    budget: &dyn CustomInvariantLeaseBudget,
) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
    let super::WorthQueryApplicationInvariantVerdict::Violation(value) = verdict else {
        return Ok(CustomInvariantVerdict::Pass);
    };
    let maximum = maximum::<Binding>().map_err(CustomInvariantExecutionError::new)?;
    let retained = (maximum as u64)
        .checked_add(Binding::IDENTITY_NAME.len() as u64)
        .ok_or_else(|| CustomInvariantExecutionError::new("refusal size overflow"))?;
    if !budget.checkpoint(1) || !budget.claim_result(retained) {
        return Err(CustomInvariantExecutionError::new(
            "refusal encoding budget refused",
        ));
    }
    let encoded = Binding::encode(&value).map_err(|denial| {
        CustomInvariantExecutionError::new(format!("refusal codec denied: {denial:?}"))
    })?;
    let AspectValue::String(InternedString::Raw(text)) = encoded else {
        return Err(CustomInvariantExecutionError::new(
            "refusal codec returned noncanonical carrier",
        ));
    };
    if text.len() > maximum {
        return Err(CustomInvariantExecutionError::new(
            "refusal exceeds declared wire bound",
        ));
    }
    Ok(CustomInvariantVerdict::Violation(
        CustomInvariantRefusal::DeclaredValue {
            value_identity: Binding::IDENTITY_NAME.to_owned(),
            bytes: text.into_bytes(),
        },
    ))
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

impl crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryCustomInvariantDenial {
    pub fn decode_refusal<Binding: ApplicationReadableScalarValueBinding>(
        &self,
    ) -> Result<Binding::Value, WorthQueryInvariantRefusalDecodeDenial> {
        match self {
            Self::Violation { refusal, .. } => decode::<Binding>(refusal),
            Self::Failure { .. } => Err(WorthQueryInvariantRefusalDecodeDenial::NoDomainRefusal),
        }
    }

    pub fn refusal_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Violation {
                refusal: CustomInvariantRefusal::DeclaredValue { bytes, .. },
                ..
            } => Some(bytes),
            Self::Violation {
                refusal: CustomInvariantRefusal::Empty,
                ..
            }
            | Self::Failure { .. } => None,
        }
    }
}
