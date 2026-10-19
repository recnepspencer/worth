//! Exact raw scalar interpretation for the Foundation operand vocabulary.
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind,
};
use worth_foundational::{
    expression_api::ExpressionValue,
    facade::{AspectValue, InternedString},
};
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
    let value = match raw {
        AspectValue::Bool(value) => ExpressionValue::bool(*value),
        AspectValue::Int8(value) => ExpressionValue::integer(i128::from(*value)),
        AspectValue::Int16(value) => ExpressionValue::integer(i128::from(*value)),
        AspectValue::Int32(value) => ExpressionValue::integer(i128::from(*value)),
        AspectValue::Int64(value) => ExpressionValue::integer(i128::from(*value)),
        AspectValue::UInt8(value) => ExpressionValue::integer(i128::from(*value)),
        AspectValue::UInt16(value) => ExpressionValue::integer(i128::from(*value)),
        AspectValue::UInt32(value) => ExpressionValue::integer(i128::from(*value)),
        AspectValue::UInt64(value) => ExpressionValue::integer(i128::from(*value)),
        AspectValue::Float32(value) => {
            ExpressionValue::float32(*value).map_err(super::expression_denial)?
        }
        AspectValue::Float64(value) => {
            ExpressionValue::float64(*value).map_err(super::expression_denial)?
        }
        AspectValue::Decimal(value) => {
            ExpressionValue::decimal(value).map_err(super::expression_denial)?
        }
        AspectValue::String(InternedString::Raw(value)) => ExpressionValue::string(value.as_str()),
        _ => return Err(invalid()),
    };
    Ok(if presence == ApplicationFieldPresence::Optional {
        ExpressionValue::some(value)
    } else {
        value
    })
}
