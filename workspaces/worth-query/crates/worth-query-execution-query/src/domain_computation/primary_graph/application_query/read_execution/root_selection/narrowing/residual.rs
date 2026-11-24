//! The residual where filter over index-selected candidates.
//!
//! Parameters bind once per request. Each candidate, in canonical identity
//! order, is a safe point: its filter fields are observed as evidence, read
//! from the pinned projection, and evaluated by Foundation's metered machine.
//! Only candidates whose program yields `true` remain.
use super::{NarrowingRead, RootPathSourceBuilder, RootSelectionWork, WHERE_FILTER_SUBJECT};
use crate::domain_computation::primary_graph::application_query::read_execution::{
    read_execution_denial, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};
use worth_foundational::expression_api::{ExpressionInputs, ExpressionValue};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_query_installation::facade::WorthQueryInstalledWhereFilter;
use worth_relational::facade::identity::EntityId;

mod evaluation;
mod row_operands;

pub fn narrow<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    filter: &WorthQueryInstalledWhereFilter,
    candidates: Vec<EntityId>,
    evidence: &mut RootPathSourceBuilder,
    work: &mut RootSelectionWork<'_>,
) -> Result<Vec<EntityId>, WorthQueryApplicationReadExecutionDenial> {
    admit_filter_fields(read, filter)?;
    let parameters = bind_parameters(read, filter)?;
    let literals = bind_literals(filter)?;
    let rows = row_operands::RowOperandReader::new(read, filter)?;
    let mut retained = Vec::new();
    retained.try_reserve_exact(candidates.len()).map_err(|_| {
        unavailable(WorthQueryApplicationReadExecutionDenialKind::ResultBufferLimitExceeded)
    })?;
    for candidate in candidates {
        work.checkpoint(WHERE_FILTER_SUBJECT)?;
        let fields = rows.read(read, candidate, evidence, work)?;
        let mut inputs = ExpressionInputs::builder(filter.program().schema());
        for (name, value) in parameters.iter().chain(&literals).chain(&fields) {
            inputs = inputs
                .bind(name, value.clone())
                .map_err(|_| projection_unavailable())?;
        }
        if evaluation::admits_row(read.execution, filter, &inputs.build(), work)? {
            retained.push(candidate);
        }
    }
    Ok(retained)
}

/// A governed query may consult a field for row presence only when its
/// disclosure contract permits that influence; public queries always may.
fn admit_filter_fields<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    filter: &WorthQueryInstalledWhereFilter,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    for field in filter.fields() {
        read.plan
            .governance
            .admit_internal_projection(
                field.field(),
                field.field_key(),
                ApplicationQueryObservableInfluence::RowPresence,
            )
            .ok_or_else(|| {
                read_execution_denial(
                    WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
                    field.field().2,
                )
            })?;
    }
    Ok(())
}

fn bind_parameters<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    filter: &WorthQueryInstalledWhereFilter,
) -> Result<Vec<(String, ExpressionValue)>, WorthQueryApplicationReadExecutionDenial> {
    filter
        .parameters()
        .iter()
        .map(|parameter| {
            let value = read
                .plan
                .parameters
                .bindings()
                .iter()
                .find(|(name, _)| name.as_ref() == parameter.parameter())
                .map(|(_, value)| value)
                .ok_or_else(projection_unavailable)?;
            let value = if parameter.is_optional() && matches!(value, AspectValue::Null) {
                ExpressionValue::none()
            } else {
                let scalar = ExpressionValue::from_aspect_value(value)
                    .map_err(|_| projection_unavailable())?;
                if parameter.is_optional() {
                    ExpressionValue::some(scalar)
                } else {
                    scalar
                }
            };
            Ok((parameter.operand().to_owned(), value))
        })
        .collect()
}

fn bind_literals(
    filter: &WorthQueryInstalledWhereFilter,
) -> Result<Vec<(String, ExpressionValue)>, WorthQueryApplicationReadExecutionDenial> {
    filter
        .literals()
        .iter()
        .map(|literal| {
            let value = ExpressionValue::from_aspect_value(literal.value())
                .map_err(|_| projection_unavailable())?;
            Ok((literal.operand().to_owned(), value))
        })
        .collect()
}

fn projection_unavailable() -> WorthQueryApplicationReadExecutionDenial {
    unavailable(WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable)
}

fn unavailable(
    kind: WorthQueryApplicationReadExecutionDenialKind,
) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(kind, WHERE_FILTER_SUBJECT)
}
