//! Exact raw scalar interpretation for the Foundation operand vocabulary.
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind,
};
use worth_foundational::{expression_api::ExpressionValue, facade::AspectValue};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldPresence, ApplicationOperationBodyRequirementSource,
};

pub(in crate::domain_computation::primary_graph) struct PreparedRequirementOperand {
    pub(in crate::domain_computation::primary_graph) value: ExpressionValue,
    pub(in crate::domain_computation::primary_graph) _reservation:
        crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
}

pub(in crate::domain_computation::primary_graph) fn field_value(
    source: &ApplicationOperationBodyRequirementSource,
    raw: Option<&AspectValue>,
) -> Result<ExpressionValue, HandlerExecutionDenial> {
    let (scalar_family, presence, presence_only) = match source {
        ApplicationOperationBodyRequirementSource::Field {
            scalar_family,
            presence,
            presence_only,
            ..
        } => (*scalar_family, *presence, *presence_only),
        ApplicationOperationBodyRequirementSource::RelatedFieldValues {
            scalar_family,
            presence,
            ..
        } => (*scalar_family, *presence, false),
        ApplicationOperationBodyRequirementSource::PrincipalIdentity { scalar_family, .. } => {
            (*scalar_family, ApplicationFieldPresence::Required, false)
        }
        _ => unreachable!("field observer accepts a scalar source"),
    };
    let invalid = || {
        HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::InvalidAuthoritativeValue,
            "requirement operand",
        ))
    };
    if raw.is_none() && presence == ApplicationFieldPresence::Required {
        return Err(HandlerExecutionDenial::new(
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                "requirement operand",
            ),
        ));
    }
    if raw.is_some_and(|raw| raw.value_family() != scalar_family) {
        return Err(invalid());
    }
    if presence_only {
        return Ok(ExpressionValue::bool(raw.is_some()));
    }
    let Some(raw) = raw else {
        return Ok(ExpressionValue::none());
    };
    let value = ExpressionValue::from_aspect_value(raw).map_err(super::expression_denial)?;
    Ok(if presence == ApplicationFieldPresence::Optional {
        ExpressionValue::some(value)
    } else {
        value
    })
}
