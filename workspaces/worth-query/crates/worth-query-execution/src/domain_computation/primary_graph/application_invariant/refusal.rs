//! The declared value codec is the sole refusal encoder and decoder.

use crate::domain_computation::vocabulary::provider_session::invariant_refusal_codec::maximum;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ValueDecodeBounds,
};
use worth_relational::facade::runtime::{
    CustomInvariantExecutionError, CustomInvariantLeaseBudget, CustomInvariantRefusal,
    CustomInvariantVerdict,
};

worth_query_declaration::worth_query_value_binding!(pub WorthQueryEmptyInvariantRefusal for () {
    identity: "worth.query.empty-invariant-refusal.v1", scalar: String,
    canonical: ValueDecodeBounds::new(4, 1, 1, 1),
});

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
