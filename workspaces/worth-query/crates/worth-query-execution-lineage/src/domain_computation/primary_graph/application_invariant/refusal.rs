//! The declared value codec is the sole refusal encoder and decoder.

use worth_foundational::facade::ScalarAspectType;
use worth_query_declaration::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ValueDecodeBounds,
};
use worth_relational::facade::runtime::{
    CustomInvariantExecutionError, CustomInvariantLeaseBudget, CustomInvariantRefusal,
    CustomInvariantVerdict,
};
pub use worth_query_execution_runtime::facade::provider_session::{
    canonical_wire_maximum, decode_refusal_payload as decode, WorthQueryInvariantRefusalDecodeDenial,
};

worth_query_declaration::worth_query_value_binding!(pub WorthQueryEmptyInvariantRefusal for () {
    identity: "worth.query.empty-invariant-refusal.v1", scalar: String,
    canonical: ValueDecodeBounds::new(4, 1, 1, 1),
});

pub fn maximum<Binding: ApplicationScalarValueBinding>() -> Result<usize, String> {
    if Binding::SCALAR_FAMILY != ScalarAspectType::String {
        return Err("invariant refusal requires a canonical application value binding".into());
    }
    canonical_wire_maximum::<Binding>()
        .ok_or_else(|| "invariant refusal requires a finite canonical wire bound".into())
}

pub fn encode<Binding: ApplicationReadableScalarValueBinding>(
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
    let Some(text) = (encoded).raw_text() else {
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
        bytes: text.as_bytes().to_vec(),
    })
}

pub fn execution_error<Binding: ApplicationReadableScalarValueBinding>(
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

pub fn preparation_error<Binding: ApplicationReadableScalarValueBinding>(
    error: super::WorthQueryApplicationInvariantError<Binding::Value>,
    budget: &dyn CustomInvariantLeaseBudget,
) -> worth_relational::facade::runtime::CustomInvariantPreparationError {
    use worth_relational::facade::runtime::CustomInvariantPreparationError as Error;
    match execution_error::<Binding>(error, budget) {
        CustomInvariantExecutionError::Cause(cause) => Error::Cause(cause),
        CustomInvariantExecutionError::Runtime(detail) => Error::Runtime(detail),
    }
}
