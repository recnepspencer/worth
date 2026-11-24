//! The residual where filter over index-selected candidates.
//!
//! Parameters bind once per request. Each candidate, in canonical identity
//! order, is a safe point: its filter fields are observed as evidence, read
//! from the pinned projection, and evaluated by Foundation's metered machine.
//! Only candidates whose program yields `true` remain.
use super::{NarrowingRead, RootPathSourceBuilder, RootSelectionWork, WHERE_FILTER_SUBJECT};
use crate::domain_computation::primary_graph::application_query::read_execution::{
    read_execution_denial, relational_read_denial, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::collections::BTreeMap;
use worth_foundational::expression_api::{ExpressionInputs, ExpressionValue};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_query_installation::facade::WorthQueryInstalledWhereFilter;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

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
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let allocation = SourceFactsAllocation::open(read.execution)
        .map_err(|cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT))?;
    let (parameters, literals) = prepare_static_operands(read, filter, &allocation, work)?;
    let rows = row_operands::RowOperandReader::new(read, filter)?;
    allocation
        .claim_slice::<EntityId>(read.execution, candidates.len())
        .map_err(|cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT))?;
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

type PreparedOperands = (Vec<(String, ExpressionValue)>, Vec<(String, ExpressionValue)>);

/// Quotes and reserves once-per-query work and memory before converting any
/// parameter or literal payload. Multiple aliases for one parameter convert
/// its payload once and share the resulting backing (`Arc<str>`).
fn prepare_static_operands<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    filter: &WorthQueryInstalledWhereFilter,
    allocation: &SourceFactsAllocation,
    work: &mut RootSelectionWork<'_>,
) -> Result<PreparedOperands, WorthQueryApplicationReadExecutionDenial> {
    let stop = |cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT);
    let mut unique_params: BTreeMap<&str, (&AspectValue, bool)> = BTreeMap::new();
    for parameter in filter.parameters() {
        let value = read
            .plan
            .parameters
            .bindings()
            .iter()
            .find(|(name, _)| name.as_ref() == parameter.parameter())
            .map(|(_, value)| value)
            .ok_or_else(projection_unavailable)?;
        unique_params
            .entry(parameter.parameter())
            .or_insert((value, parameter.is_optional()));
    }

    let mut total_work = 0_u64;
    let mut total_bytes = 0_u64;
    for (value, is_optional) in unique_params.values() {
        let payload = payload_bytes(value, *is_optional);
        total_work = total_work
            .checked_add(1)
            .and_then(|n| n.checked_add(payload))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        let alloc = payload
            .checked_add((size_of::<ExpressionValue>() + 2 * size_of::<usize>()) as u64)
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
        total_bytes = total_bytes
            .checked_add(alloc)
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    }
    for parameter in filter.parameters() {
        let slot_bytes = (parameter.operand().len()
            + size_of::<(String, ExpressionValue)>()
            + if parameter.is_optional() {
                size_of::<ExpressionValue>()
            } else {
                0
            }) as u64;
        total_work = total_work
            .checked_add(1)
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        total_bytes = total_bytes
            .checked_add(slot_bytes)
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    }
    for literal in filter.literals() {
        let payload = payload_bytes(literal.value(), false);
        let slot_bytes = (literal.operand().len()
            + size_of::<(String, ExpressionValue)>()
            + size_of::<ExpressionValue>()
            + 2 * size_of::<usize>()) as u64;
        total_work = total_work
            .checked_add(1)
            .and_then(|n| n.checked_add(payload))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        total_bytes = total_bytes
            .checked_add(slot_bytes)
            .and_then(|n| n.checked_add(payload))
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    }

    if total_work > 0 {
        SourceFactsAllocation::work(read.execution, total_work).map_err(stop)?;
        work.charge_residual_filter(
            usize::try_from(total_work).unwrap_or(usize::MAX),
            WHERE_FILTER_SUBJECT,
        )?;
    }
    if total_bytes > 0 {
        allocation.claim(read.execution, total_bytes).map_err(stop)?;
    }

    let mut converted: BTreeMap<&str, ExpressionValue> = BTreeMap::new();
    for (name, (value, is_optional)) in unique_params {
        let expr_value = if is_optional && matches!(value, AspectValue::Null) {
            ExpressionValue::none()
        } else {
            let scalar = ExpressionValue::from_aspect_value(value)
                .map_err(|_| projection_unavailable())?;
            if is_optional {
                ExpressionValue::some(scalar)
            } else {
                scalar
            }
        };
        converted.insert(name, expr_value);
    }

    let mut parameters = Vec::with_capacity(filter.parameters().len());
    for parameter in filter.parameters() {
        let value = converted
            .get(parameter.parameter())
            .ok_or_else(projection_unavailable)?;
        parameters.push((parameter.operand().to_owned(), value.clone()));
    }
    let mut literals = Vec::with_capacity(filter.literals().len());
    for literal in filter.literals() {
        let value = ExpressionValue::from_aspect_value(literal.value())
            .map_err(|_| projection_unavailable())?;
        literals.push((literal.operand().to_owned(), value));
    }
    Ok((parameters, literals))
}

fn payload_bytes(value: &AspectValue, is_optional: bool) -> u64 {
    if is_optional && matches!(value, AspectValue::Null) {
        0
    } else {
        u64::try_from(value.semantic_byte_width()).unwrap_or(u64::MAX)
    }
}

fn projection_unavailable() -> WorthQueryApplicationReadExecutionDenial {
    unavailable(WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable)
}

fn unavailable(
    kind: WorthQueryApplicationReadExecutionDenialKind,
) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(kind, WHERE_FILTER_SUBJECT)
}
