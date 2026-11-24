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
    let rows = row_operands::RowOperandReader::new(read, filter, &allocation, work)?;
    allocation
        .claim_slice::<EntityId>(read.execution, candidates.len())
        .map_err(|cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT))?;
    let mut retained = Vec::new();
    retained.try_reserve_exact(candidates.len()).map_err(|_| {
        unavailable(WorthQueryApplicationReadExecutionDenialKind::ResultBufferLimitExceeded)
    })?;
    let builder_schema_bytes =
        ExpressionInputs::scalar_builder_schema_and_conform_bytes(filter.program().schema())
            .ok_or_else(projection_unavailable)
            .and_then(|bytes| {
                u64::try_from(bytes).map_err(|_| {
                    relational_read_denial(Cause::ChargedBytesOverflow, WHERE_FILTER_SUBJECT)
                })
            })?;
    let static_input_bytes = operands_input_bytes(&parameters)
        .and_then(|n| n.checked_add(operands_input_bytes(&literals)?))
        .and_then(|n| n.checked_add(builder_schema_bytes))
        .ok_or_else(|| relational_read_denial(Cause::ChargedBytesOverflow, WHERE_FILTER_SUBJECT))?;
    let total_operands = parameters
        .len()
        .checked_add(literals.len())
        .and_then(|n| n.checked_add(filter.fields().len()))
        .ok_or_else(|| relational_read_denial(Cause::WorkCounterOverflow, WHERE_FILTER_SUBJECT))?;
    let schema_len = filter.program().schema().operands().len();
    let per_bind_work = log2_ceil(total_operands)
        .checked_add(log2_ceil(schema_len))
        .and_then(|n| n.checked_add(2))
        .ok_or_else(|| relational_read_denial(Cause::WorkCounterOverflow, WHERE_FILTER_SUBJECT))?;
    let input_build_work = (total_operands as u64)
        .checked_mul(per_bind_work)
        .and_then(|n| n.checked_add(schema_len as u64))
        .and_then(|n| n.checked_add(builder_schema_bytes))
        .ok_or_else(|| relational_read_denial(Cause::WorkCounterOverflow, WHERE_FILTER_SUBJECT))?;
    for candidate in candidates {
        work.checkpoint(WHERE_FILTER_SUBJECT)?;
        let row_alloc = SourceFactsAllocation::open(read.execution)
            .map_err(|cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT))?;
        let fields = rows.read(read, candidate, evidence, &row_alloc, work)?;
        let field_input_bytes = operands_input_bytes(&fields).ok_or_else(|| {
            relational_read_denial(Cause::ChargedBytesOverflow, WHERE_FILTER_SUBJECT)
        })?;
        let row_input_bytes = static_input_bytes
            .checked_add(field_input_bytes)
            .ok_or_else(|| {
                relational_read_denial(Cause::ChargedBytesOverflow, WHERE_FILTER_SUBJECT)
            })?;
        charge_work(read.execution, work, input_build_work)?;
        row_alloc
            .claim_tree_copy::<(Box<str>, ExpressionValue)>(read.execution, total_operands)
            .map_err(|cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT))?;
        row_alloc
            .claim_arc::<BTreeMap<Box<str>, ExpressionValue>>(read.execution)
            .map_err(|cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT))?;
        if row_input_bytes > 0 {
            row_alloc
                .claim(read.execution, row_input_bytes)
                .map_err(|cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT))?;
        }
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

type PreparedOperands = (
    Vec<(String, ExpressionValue)>,
    Vec<(String, ExpressionValue)>,
);

/// Charges lookup and dedup work upfront, reserves scratch and payload memory
/// before any allocation or conversion, and converts each unique parameter
/// once so multiple operand aliases share its backing (`Arc<str>`).
fn prepare_static_operands<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    filter: &WorthQueryInstalledWhereFilter,
    allocation: &SourceFactsAllocation,
    work: &mut RootSelectionWork<'_>,
) -> Result<PreparedOperands, WorthQueryApplicationReadExecutionDenial> {
    let stop = |cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT);
    let bindings = read.plan.parameters.bindings();
    let filter_params = filter.parameters();
    let filter_literals = filter.literals();
    let param_count = filter_params.len();
    let literal_count = filter_literals.len();

    // Charge O(F * (log P + log F) + L) lookup/sort/dedup work BEFORE scanning.
    let lookup_factor = log2_ceil(bindings.len())
        .checked_add(log2_ceil(param_count))
        .and_then(|n| n.checked_add(2))
        .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
    let scan_work = (param_count as u64)
        .checked_mul(lookup_factor)
        .and_then(|n| n.checked_add(literal_count as u64))
        .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
    charge_work(read.execution, work, scan_work)?;

    // Reserve index-resolution scratch before allocating any Vec.
    let scratch_bytes = param_count
        .checked_mul(2 * size_of::<(usize, bool)>() + size_of::<(usize, ExpressionValue)>())
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    if scratch_bytes > 0 {
        allocation
            .claim(read.execution, scratch_bytes)
            .map_err(stop)?;
    }
    let mut resolved = try_vec_exact(param_count)?;
    let mut unique = try_vec_exact(param_count)?;
    for parameter in filter_params {
        let index = bindings
            .binary_search_by(|(name, _)| name.as_ref().cmp(parameter.parameter()))
            .map_err(|_| projection_unavailable())?;
        resolved.push((index, parameter.is_optional()));
        unique.push((index, parameter.is_optional()));
    }
    unique.sort_unstable_by_key(|&(index, _)| index);
    unique.dedup_by_key(|entry| entry.0);

    let mut payload_work = 0_u64;
    let mut payload_alloc = 0_u64;
    for &(index, is_optional) in &unique {
        let payload = payload_bytes(&bindings[index].1, is_optional);
        let alloc = payload
            .checked_add((size_of::<ExpressionValue>() + 2 * size_of::<usize>()) as u64)
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
        payload_work = payload_work
            .checked_add(1)
            .and_then(|n| n.checked_add(payload))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        payload_alloc = payload_alloc
            .checked_add(alloc)
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    }
    for parameter in filter_params {
        let slot_bytes = (parameter.operand().len()
            + size_of::<(String, ExpressionValue)>()
            + if parameter.is_optional() {
                size_of::<ExpressionValue>()
            } else {
                0
            }) as u64;
        payload_alloc = payload_alloc
            .checked_add(slot_bytes)
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    }
    for literal in filter_literals {
        let payload = payload_bytes(literal.value(), false);
        let slot_bytes = (literal.operand().len()
            + size_of::<(String, ExpressionValue)>()
            + size_of::<ExpressionValue>()
            + 2 * size_of::<usize>()) as u64;
        payload_work = payload_work
            .checked_add(1)
            .and_then(|n| n.checked_add(payload))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        payload_alloc = payload_alloc
            .checked_add(slot_bytes)
            .and_then(|n| n.checked_add(payload))
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    }

    charge_work(read.execution, work, payload_work)?;
    if payload_alloc > 0 {
        allocation
            .claim(read.execution, payload_alloc)
            .map_err(stop)?;
    }

    let mut converted: Vec<(usize, ExpressionValue)> = try_vec_exact(unique.len())?;
    for &(index, is_optional) in &unique {
        let expr_value = convert_parameter_value(&bindings[index].1, is_optional)?;
        converted.push((index, expr_value));
    }
    let mut parameters = try_vec_exact(param_count)?;
    for (parameter, &(index, _)) in filter_params.iter().zip(&resolved) {
        let pos = converted
            .binary_search_by_key(&index, |&(idx, _)| idx)
            .map_err(|_| projection_unavailable())?;
        parameters.push((parameter.operand().to_owned(), converted[pos].1.clone()));
    }
    let mut literals = try_vec_exact(literal_count)?;
    for literal in filter_literals {
        let value = ExpressionValue::from_aspect_value(literal.value())
            .map_err(|_| projection_unavailable())?;
        literals.push((literal.operand().to_owned(), value));
    }
    Ok((parameters, literals))
}

pub(super) fn charge_work(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    work: &mut RootSelectionWork<'_>,
    units: u64,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    if units == 0 {
        return Ok(());
    }
    SourceFactsAllocation::work(execution, units)
        .map_err(|cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT))?;
    work.charge_residual_filter(
        usize::try_from(units).unwrap_or(usize::MAX),
        WHERE_FILTER_SUBJECT,
    )
}

pub(super) fn log2_ceil(len: usize) -> u64 {
    if len <= 1 {
        1
    } else {
        u64::from(usize::BITS - (len - 1).leading_zeros()) + 1
    }
}

pub(super) fn try_vec_exact<T>(
    len: usize,
) -> Result<Vec<T>, WorthQueryApplicationReadExecutionDenial> {
    let mut vec = Vec::new();
    vec.try_reserve_exact(len).map_err(|_| {
        unavailable(WorthQueryApplicationReadExecutionDenialKind::ResultBufferLimitExceeded)
    })?;
    Ok(vec)
}

fn operands_input_bytes(operands: &[(String, ExpressionValue)]) -> Option<u64> {
    let mut total = 0_u64;
    let clone_box = size_of::<ExpressionValue>() as u64;
    for (name, _) in operands {
        let name_bytes = u64::try_from(name.len()).ok()?;
        total = total.checked_add(name_bytes)?.checked_add(clone_box)?;
    }
    Some(total)
}

fn convert_parameter_value(
    value: &AspectValue,
    is_optional: bool,
) -> Result<ExpressionValue, WorthQueryApplicationReadExecutionDenial> {
    if is_optional && matches!(value, AspectValue::Null) {
        return Ok(ExpressionValue::none());
    }
    let scalar = ExpressionValue::from_aspect_value(value).map_err(|_| projection_unavailable())?;
    Ok(if is_optional {
        ExpressionValue::some(scalar)
    } else {
        scalar
    })
}

pub(super) fn payload_bytes(value: &AspectValue, is_optional: bool) -> u64 {
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
