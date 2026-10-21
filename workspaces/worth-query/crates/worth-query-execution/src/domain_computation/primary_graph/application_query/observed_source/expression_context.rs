//! Native disclosed values and their recursive source footprint stay together.
//! This interprets the materialized tree; it never reads/traverses the graph.
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotDenialKind as Denial;
use crate::domain_computation::primary_graph::application_query::projection::WorthQueryApplicationProjectionNode;
use worth_foundational::expression_api::{ExpressionSchema, ExpressionValue};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryExpressionSnapshot,
};
use worth_query_installation::facade::{
    ApplicationFieldPresence, WorthQueryInstalledGraphReadContract,
};

pub(in crate::domain_computation::primary_graph::application_query) struct RetainedExpressionContext
{
    snapshot: ApplicationQueryExpressionSnapshot,
}

impl RetainedExpressionContext {
    pub(super) fn schema(&self) -> &ExpressionSchema {
        self.snapshot.schema()
    }
    pub(super) fn value(&self) -> &ExpressionValue {
        self.snapshot.value()
    }
    pub(super) fn snapshot(&self) -> &ApplicationQueryExpressionSnapshot {
        &self.snapshot
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn retain(
        contract: &WorthQueryInstalledGraphReadContract,
        schema: Option<&ExpressionSchema>,
        read: &crate::domain_computation::primary_graph::application_query::read_execution::RawOneShotRows,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<Self, Denial> {
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
        Ok(Self {
            snapshot: ApplicationQueryExpressionSnapshot::from_untrusted_parts(schema, value),
        })
    }
}

fn project(
    node: &WorthQueryApplicationProjectionNode,
    path: &str,
    contract: &WorthQueryInstalledGraphReadContract,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<ExpressionValue, Denial> {
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
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
            (ApplicationFieldPresence::Required, Some(value)) => {
                ExpressionValue::from_aspect_value(value)
                    .map_err(|_| Denial::ExpressionContextUnavailable)?
            }
            (ApplicationFieldPresence::Required, None) => {
                return Err(Denial::ExpressionContextUnavailable)
            }
            (ApplicationFieldPresence::Optional, Some(value)) => ExpressionValue::some(
                ExpressionValue::from_aspect_value(value)
                    .map_err(|_| Denial::ExpressionContextUnavailable)?,
            ),
            (ApplicationFieldPresence::Optional, None) => ExpressionValue::none(),
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
