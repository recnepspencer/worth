//! Native disclosed values and their recursive source footprint stay together.
//! This interprets the materialized tree; it never reads/traverses the graph.
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotDenialKind as Denial;
use crate::domain_computation::primary_graph::application_query::{
    projection::WorthQueryApplicationProjectionNode,
    resource_lifecycle::{
        WorthQueryApplicationResultBufferReservation, WorthQueryRetainedSourceCharge,
    },
};
use worth_foundational::expression_api::{ExpressionSchema, ExpressionValue};
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_installation::facade::{
    ApplicationFieldPresence, WorthQueryInstalledGraphReadContract,
};

pub(in crate::domain_computation::primary_graph::application_query) struct RetainedExpressionContext
{
    schema: ExpressionSchema,
    value: ExpressionValue,
    _charge: WorthQueryRetainedSourceCharge,
}

impl RetainedExpressionContext {
    pub(super) fn schema(&self) -> &ExpressionSchema {
        &self.schema
    }
    pub(super) fn value(&self) -> &ExpressionValue {
        &self.value
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn retain(
        contract: &WorthQueryInstalledGraphReadContract,
        schema: Option<&ExpressionSchema>,
        read: &crate::domain_computation::primary_graph::application_query::read_execution::RawOneShotRows,
        buffer: &WorthQueryApplicationResultBufferReservation,
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
        // The read already bounded the native tree. Retained expression values
        // use Foundation's conservative backing size; the type vocabulary
        // is bounded by the exact installed query, not by caller-provided rows.
        let bytes = value
            .retained_allocation_bytes()
            .checked_add(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())
            .and_then(|bytes| bytes.checked_add(schema_retained_bytes(contract)?))
            .ok_or(Denial::ResultBufferLimitExceeded)?;
        let charge = buffer
            .claim_retained_source(bytes)
            .map_err(|()| Denial::ResultBufferLimitExceeded)?;
        Ok(Self {
            schema,
            value,
            _charge: charge,
        })
    }
}

fn schema_retained_bytes(contract: &WorthQueryInstalledGraphReadContract) -> Option<usize> {
    // Conservative custody for the retained installed vocabulary, including
    // lookup nodes, field names and nominal type names. No schema is rebuilt.
    contract
        .projections()
        .iter()
        .map(|field| (field.result_path(), field.output_name()))
        .chain(
            contract
                .relations()
                .iter()
                .map(|relation| (relation.result_path(), relation.output_name())),
        )
        .try_fold(1024_usize, |bytes, (path, name)| {
            bytes
                .checked_add(1024)?
                .checked_add(path.len().checked_mul(4)?)?
                .checked_add(name.len().checked_mul(4)?)
        })
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
