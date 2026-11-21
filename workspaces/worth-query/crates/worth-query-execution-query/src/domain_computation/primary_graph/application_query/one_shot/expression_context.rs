//! Expression-context projection from raw one-shot rows into `RetainedExpressionContext`.

use crate::domain_computation::primary_graph::application_query::{
    observed_source::RetainedExpressionContext,
    projection::WorthQueryApplicationProjectionNode,
    read_execution::RawOneShotRows,
    WorthQueryApplicationOneShotDenialKind as Denial,
};
use worth_foundational::expression_api::{ExpressionSchema, ExpressionValue};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryExpressionSnapshot,
    ApplicationQueryResultFieldPresence,
};
use worth_query_installation::facade::WorthQueryInstalledGraphReadContract;

pub fn retain_expression_context(
    contract: &WorthQueryInstalledGraphReadContract,
    schema: Option<&ExpressionSchema>,
    read: &RawOneShotRows,
    request: &WorthQueryRequestScope,
) -> Result<RetainedExpressionContext, Denial> {
    let ([row], [footprint]) = (read.rows.raw_rows(), read.source_footprints.as_slice()) else {
        return Err(Denial::ExpressionContextUnavailable);
    };
    if contract.cardinality() != ApplicationQueryCardinality::ExactlyOne {
        return Err(Denial::ExpressionContextUnavailable);
    }
    if read.has_more || !footprint.complete {
        return Err(Denial::ExpressionContextIncomplete);
    }
    let schema = schema.ok_or(Denial::ExpressionContextUnavailable)?.clone();
    let value = project(row, "root", contract, request)?;
    Ok(RetainedExpressionContext::from_snapshot(
        ApplicationQueryExpressionSnapshot::from_untrusted_parts(schema, value),
    ))
}

fn project(
    node: &WorthQueryApplicationProjectionNode,
    path: &str,
    contract: &WorthQueryInstalledGraphReadContract,
    request: &WorthQueryRequestScope,
) -> Result<ExpressionValue, Denial> {
    if let Some(interruption) = request.interruption() {
        return Err(match interruption {
            WorthQueryRequestInterruption::Cancelled => Denial::Cancelled,
            WorthQueryRequestInterruption::DeadlineExceeded => Denial::DeadlineExceeded,
        });
    }
    if !node.source_dependencies_complete() {
        return Err(Denial::ExpressionContextIncomplete);
    }
    let mut fields = Vec::new();
    for field in contract
        .projections()
        .iter()
        .filter(|field| field.parent_path() == path)
    {
        let value = node.field(field.slot_type()).map(|field| field.value());
        fields.push(match (field.presence(), value) {
            (ApplicationQueryResultFieldPresence::Required, Some(value)) => {
                ExpressionValue::from_aspect_value(value)
                    .map_err(|_| Denial::ExpressionContextUnavailable)?
            }
            (ApplicationQueryResultFieldPresence::Required, None) => {
                return Err(Denial::ExpressionContextUnavailable)
            }
            (
                ApplicationQueryResultFieldPresence::Optional
                | ApplicationQueryResultFieldPresence::OptionalAspect,
                Some(value),
            ) => ExpressionValue::some(
                ExpressionValue::from_aspect_value(value)
                    .map_err(|_| Denial::ExpressionContextUnavailable)?,
            ),
            (
                ApplicationQueryResultFieldPresence::Optional
                | ApplicationQueryResultFieldPresence::OptionalAspect,
                None,
            ) => ExpressionValue::none(),
        });
    }
    for relation in contract
        .relations()
        .iter()
        .filter(|relation| relation.parent_path() == path)
    {
        let rows = node
            .relation(relation.slot_type())
            .ok_or(Denial::ExpressionContextUnavailable)?
            .rows();
        let mut values = rows
            .iter()
            .map(|row| project(row, relation.result_path(), contract, request))
            .collect::<Result<Vec<_>, _>>()?;
        fields.push(match relation.cardinality() {
            ApplicationQueryCardinality::Many => ExpressionValue::list(values),
            ApplicationQueryCardinality::OptionalOne if values.is_empty() => {
                ExpressionValue::none()
            }
            ApplicationQueryCardinality::OptionalOne if values.len() == 1 => {
                ExpressionValue::some(values.pop().ok_or(Denial::ExpressionContextUnavailable)?)
            }
            ApplicationQueryCardinality::ExactlyOne if values.len() == 1 => {
                values.pop().ok_or(Denial::ExpressionContextUnavailable)?
            }
            _ => return Err(Denial::ExpressionContextUnavailable),
        });
    }
    Ok(ExpressionValue::record(fields))
}
