//! Pre-admit the finite validation leaf families, without decoding the body.
use super::*;
use crate::transactions::data::{
    ValidationLeafMemory, ValidationReadmissionMemory, ValidationTouchedMemory,
};
mod visit;

pub(in super::super::super) fn prepare_canonical_readmission(
    canonical: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(ValidationReadmissionMemory, u64), Refusal> {
    let semantic = field(canonical, "semantic_result", request)?;
    prepare_semantic_readmission(semantic, request)
}

pub(in super::super::super) fn prepare_semantic_readmission(
    semantic: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(ValidationReadmissionMemory, u64), Refusal> {
    let validation = field(semantic, "validation", request)?;
    let (mut rules, mut touched, mut projected) = (0usize, 0usize, 0usize);
    let (mut rule_bytes_total, mut touched_bytes_total, mut projected_bytes_total) =
        (0u64, 0u64, 0u64);
    visit::validation(
        validation,
        request,
        &mut |text| {
            rules = rules.checked_add(1).ok_or_else(malformed)?;
            sum(&mut rule_bytes_total, rule_bytes(text, request)?.retained)
        },
        &mut |summary| {
            touched = touched.checked_add(1).ok_or_else(malformed)?;
            sum(&mut touched_bytes_total, touched_bytes(summary, request)?)
        },
        &mut |value| {
            projected = projected.checked_add(1).ok_or_else(malformed)?;
            sum(&mut projected_bytes_total, member(value, request)?.retained)
        },
    )?;
    let tree_bytes = layout::<ValidationLeafMemory>(rules)?
        .checked_add(layout::<ValidationTouchedMemory>(touched)?)
        .and_then(|n| n.checked_add(layout::<ValidationLeafMemory>(projected).ok()?))
        .ok_or_else(malformed)?;
    let tree_memory = request
        .reserve_retained_memory(tree_bytes)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let mut rules = Vec::with_capacity(rules);
    let mut touched = Vec::with_capacity(touched);
    let mut projected = Vec::with_capacity(projected);
    visit::validation(
        validation,
        request,
        &mut |text| {
            rules.push(leaf(
                text.text(request)?.len(),
                rule_bytes(text, request)?.retained,
                request,
            )?);
            Ok(())
        },
        &mut |summary| {
            let entities = field(summary, "visible_entity_ids", request)?;
            let relations = field(summary, "visible_relation_ids", request)?;
            let partitions = field(summary, "touched_partition_ids", request)?;
            touched.push(ValidationTouchedMemory {
                entities: leaf(
                    count(entities, request)?,
                    touched_entities(entities, request)?.retained,
                    request,
                )?,
                relations: leaf(
                    count(relations, request)?,
                    touched_relations(relations, request)?.retained,
                    request,
                )?,
                partitions: leaf(
                    count(partitions, request)?,
                    touched_partitions(partitions, request)?.retained,
                    request,
                )?,
            });
            Ok(())
        },
        &mut |value| {
            let coordinate = field(value, "Projected", request)?;
            let domain = field(coordinate, "domain", request)?;
            let case_length = field(coordinate, "case", request)?.text(request)?.len();
            let length = field(domain, "schema", request)?
                .text(request)?
                .len()
                .checked_add(field(domain, "name", request)?.text(request)?.len())
                .and_then(|n| n.checked_add(case_length))
                .ok_or_else(malformed)?;
            projected.push(leaf(length, member(value, request)?.retained, request)?);
            Ok(())
        },
    )?;
    Ok((
        ValidationReadmissionMemory {
            rules,
            touched,
            projected,
            _tree_memory: tree_memory,
        },
        [rule_bytes_total, touched_bytes_total, projected_bytes_total]
            .into_iter()
            .try_fold(0u64, |sum, bytes| sum.checked_add(bytes))
            .ok_or_else(malformed)?,
    ))
}
fn rule_bytes(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_text(value, request)?;
    add(
        &mut bytes,
        crate::validation::data::CustomInvariantRuleId::control_bytes().ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
fn touched_bytes(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<u64, Refusal> {
    let entities =
        touched_entities(field(value, "visible_entity_ids", request)?, request)?.retained;
    let relations =
        touched_relations(field(value, "visible_relation_ids", request)?, request)?.retained;
    let partitions =
        touched_partitions(field(value, "touched_partition_ids", request)?, request)?.retained;
    entities
        .checked_add(relations)
        .and_then(|n| n.checked_add(partitions))
        .ok_or_else(malformed)
}
fn count(value: BorrowedPayload<'_>, request: &ExecutionRequest<'_, '_>) -> Result<usize, Refusal> {
    value.elements(request, |_| Ok(()))
}
fn layout<T>(count: usize) -> Result<u64, Refusal> {
    u64::try_from(
        std::alloc::Layout::array::<T>(count)
            .map_err(|_| malformed())?
            .size(),
    )
    .map_err(|_| malformed())
}
fn sum(total: &mut u64, bytes: u64) -> Result<(), Refusal> {
    *total = total.checked_add(bytes).ok_or_else(malformed)?;
    Ok(())
}
fn leaf(
    length: usize,
    bytes: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ValidationLeafMemory, Refusal> {
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    Ok(ValidationLeafMemory { length, memory })
}
