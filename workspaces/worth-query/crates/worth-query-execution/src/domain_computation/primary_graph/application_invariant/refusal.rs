//! The declared value codec is the sole refusal encoder and decoder.

use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};
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

pub(super) fn maximum<Binding: ApplicationScalarValueBinding>() -> Result<usize, String> {
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

pub(super) fn encode<Binding: ApplicationReadableScalarValueBinding>(
    verdict: super::WorthQueryApplicationInvariantVerdict<Binding::Value>,
    budget: &dyn CustomInvariantLeaseBudget,
) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
    let super::WorthQueryApplicationInvariantVerdict::Violation(value) = verdict else {
        return Ok(CustomInvariantVerdict::Pass);
    };
    encode_value::<Binding>(&value, budget).map(CustomInvariantVerdict::Violation)
}

fn encode_value<Binding: ApplicationReadableScalarValueBinding>(
    value: &Binding::Value,
    budget: &dyn CustomInvariantLeaseBudget,
) -> Result<CustomInvariantRefusal, CustomInvariantExecutionError> {
    let maximum = maximum::<Binding>().map_err(CustomInvariantExecutionError::new)?;
    let retained = (maximum as u64)
        .checked_add(Binding::IDENTITY_NAME.len() as u64)
        .ok_or_else(|| CustomInvariantExecutionError::new("refusal size overflow"))?;
    if !budget.checkpoint(1) || !budget.claim_result(retained) {
        return Err(CustomInvariantExecutionError::new(
            "refusal encoding budget refused",
        ));
    }
    let encoded = Binding::encode(value).map_err(|denial| {
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
    Ok(CustomInvariantRefusal::DeclaredValue {
        value_identity: Binding::IDENTITY_NAME.to_owned(),
        bytes: text.into_bytes(),
    })
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

impl crate::domain_computation::WorthQueryCustomInvariantDenial {
    pub fn decode_refusal<Binding: ApplicationReadableScalarValueBinding>(
        &self,
    ) -> Result<Binding::Value, WorthQueryInvariantRefusalDecodeDenial> {
        decode::<Binding>(
            self.refusal_payload()
                .ok_or(WorthQueryInvariantRefusalDecodeDenial::NoDomainRefusal)?,
        )
    }

    pub fn refusal_bytes(&self) -> Option<&[u8]> {
        match self.refusal_payload()? {
            CustomInvariantRefusal::DeclaredValue { bytes, .. } => Some(bytes),
            CustomInvariantRefusal::Empty => None,
        }
    }

    pub(crate) fn refusal_payload(&self) -> Option<&CustomInvariantRefusal> {
        match self {
            Self::Violation { refusal, .. } => Some(refusal),
            Self::Failure { cause, .. } => match cause.as_deref()? {
                worth_relational::facade::runtime::CustomInvariantFailureCause::DeclaredValue(
                    refusal,
                ) => Some(refusal),
                _ => None,
            },
        }
    }
}

pub(super) fn execution_error<Binding: ApplicationReadableScalarValueBinding>(
    error: super::WorthQueryApplicationInvariantError<Binding::Value>,
    budget: &dyn CustomInvariantLeaseBudget,
) -> CustomInvariantExecutionError {
    match error.into_cause() {
        Ok(value) => match encode_value::<Binding>(&value, budget) {
            Ok(cause) => CustomInvariantExecutionError::Cause(Box::new(
                worth_relational::facade::runtime::CustomInvariantFailureCause::DeclaredValue(
                    cause,
                ),
            )),
            Err(error) => error,
        },
        Err(error) => system_error(error),
    }
}

fn system_error(error: super::hook_error::SystemInvariantError) -> CustomInvariantExecutionError {
    CustomInvariantExecutionError::Cause(Box::new(error))
}

pub(super) fn preparation_error<Binding: ApplicationReadableScalarValueBinding>(
    error: super::WorthQueryApplicationInvariantError<Binding::Value>,
    budget: &dyn CustomInvariantLeaseBudget,
) -> worth_relational::facade::runtime::CustomInvariantPreparationError {
    use worth_relational::facade::runtime::CustomInvariantPreparationError as Error;
    match execution_error::<Binding>(error, budget) {
        CustomInvariantExecutionError::Cause(cause) => Error::Cause(cause),
        CustomInvariantExecutionError::Runtime(detail) => Error::Runtime(detail),
    }
}
