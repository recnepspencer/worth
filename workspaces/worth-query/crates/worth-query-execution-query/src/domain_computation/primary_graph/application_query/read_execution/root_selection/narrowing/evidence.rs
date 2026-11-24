//! Narrowing observations join the selection's own result-set evidence.
//!
//! Every merge scratch buffer, cloned field-revision/adjacency payload, and
//! O(n log n) allocation-free unstable sort/dedup step is preflighted and
//! charged through `SourceFactsAllocation` before allocating or cloning.
use super::residual::{log2_ceil, try_vec_exact};
use super::RootPathSourceBuilder;
use crate::domain_computation::primary_graph::application_query::{
    observed_source::{
        WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
        WorthQueryObservedRootSelection,
    },
    read_execution::{
        relational_read_denial, root_selection::path_union::traversal_denial,
        WorthQueryApplicationReadExecutionDenial,
    },
};
use crate::domain_computation::primary_graph::source_facts::{SourceFacts, SourceFactsAllocation};
use std::sync::Arc;
use worth_relational::facade::{
    identity::EntityId, runtime::RelationalAdjacencyDirection,
    transactions::RelationalExecutionDenialCause as Cause,
};

/// One canonical set witness: the selector's reads plus every narrowing
/// read (entities, field revisions, and adjacency structural revisions),
/// deduplicated in canonical key order with base-first precedence.
pub fn merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    base: Option<Arc<WorthQueryObservedRootSelection>>,
    narrowing: RootPathSourceBuilder,
    subject: &str,
) -> Result<Option<Arc<WorthQueryObservedRootSelection>>, WorthQueryApplicationReadExecutionDenial>
{
    let Some(base) = base else {
        return Ok(None);
    };
    if base.orbit.is_some() || base.components.is_some() || base.managed_index.is_some() {
        return Err(traversal_denial(subject));
    }
    let narrowing = narrowing.finish(execution)?;
    let stop = |cause| relational_read_denial(cause, subject);

    let total_entities = checked_add(base.entities.len(), narrowing.entities.len(), subject)?;
    let total_aspects = checked_add(base.aspects.len(), narrowing.aspects.len(), subject)?;
    let total_adjacencies =
        checked_add(base.adjacencies.len(), narrowing.adjacencies.len(), subject)?;
    let aspect_clone_bytes = field_revision_bytes(&base.aspects, &narrowing.aspects)
        .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    let adj_clone_bytes = adjacency_revision_bytes(&base.adjacencies, &narrowing.adjacencies)
        .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    let merge_work = merge_work_units(
        total_entities,
        total_aspects,
        total_adjacencies,
        aspect_clone_bytes,
        adj_clone_bytes,
    )
    .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
    if merge_work > 0 {
        SourceFactsAllocation::work(execution, merge_work).map_err(stop)?;
    }

    let scratch = SourceFactsAllocation::open(execution).map_err(stop)?;
    claim_scratch_memory(
        execution,
        &scratch,
        total_entities,
        total_aspects,
        total_adjacencies,
        aspect_clone_bytes,
        adj_clone_bytes,
        subject,
    )?;

    let mut tagged_aspects =
        merge_tagged_aspects(&base.aspects, &narrowing.aspects, total_aspects)?;
    let mut tagged_adjacencies =
        merge_tagged_adjacencies(&base.adjacencies, &narrowing.adjacencies, total_adjacencies)?;
    let mut scratch_entities = merge_entities(&base.entities, &narrowing.entities, total_entities)?;

    // Shrink/materialize exact-capacity output vectors under the retained
    // allocation (or scratch when unattached) so retained claims match actual
    // vector capacities with zero hidden over-allocation.
    let target_alloc = base.allocation.as_ref().unwrap_or(&scratch);
    let kept_aspect_bytes = tagged_aspects
        .iter()
        .try_fold(0_u64, |acc, (_, _, r)| {
            acc.checked_add(u64::try_from(r.owned_capacity_bytes()).ok()?)
        })
        .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    let kept_adj_bytes = tagged_adjacencies
        .iter()
        .try_fold(0_u64, |acc, (_, _, r)| {
            acc.checked_add(u64::try_from(r.owned_capacity_bytes()).ok()?)
        })
        .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;

    let entities = exact_copy_entities(execution, target_alloc, &mut scratch_entities, subject)?;
    let aspects = drain_exact(
        execution,
        target_alloc,
        &mut tagged_aspects,
        kept_aspect_bytes,
        subject,
    )?;
    let adjacencies = drain_exact(
        execution,
        target_alloc,
        &mut tagged_adjacencies,
        kept_adj_bytes,
        subject,
    )?;
    drop(scratch_entities);
    drop(tagged_aspects);
    drop(tagged_adjacencies);
    drop(scratch);

    let facts = SourceFacts {
        entities,
        aspects,
        adjacencies,
        walks: base.walks.clone(),
        derived: base.derived.clone(),
        allocation: base.allocation.clone(),
    };
    facts
        .admit_container::<WorthQueryObservedRootSelection>(execution)
        .map_err(stop)?;
    Ok(Some(Arc::new(WorthQueryObservedRootSelection::new(
        facts, None,
    ))))
}

fn checked_add(
    left: usize,
    right: usize,
    subject: &str,
) -> Result<usize, WorthQueryApplicationReadExecutionDenial> {
    left.checked_add(right)
        .ok_or_else(|| relational_read_denial(Cause::WorkCounterOverflow, subject))
}

fn field_revision_bytes(
    left: &[WorthQueryObservedFieldRevision],
    right: &[WorthQueryObservedFieldRevision],
) -> Option<u64> {
    left.iter().chain(right).try_fold(0_u64, |acc, rev| {
        acc.checked_add(u64::try_from(rev.owned_capacity_bytes()).ok()?)
    })
}

fn adjacency_revision_bytes(
    left: &[WorthQueryObservedAdjacencyRevision],
    right: &[WorthQueryObservedAdjacencyRevision],
) -> Option<u64> {
    left.iter().chain(right).try_fold(0_u64, |acc, rev| {
        acc.checked_add(u64::try_from(rev.owned_capacity_bytes()).ok()?)
    })
}

fn merge_work_units(
    entities: usize,
    aspects: usize,
    adjacencies: usize,
    aspect_bytes: u64,
    adj_bytes: u64,
) -> Option<u64> {
    let sort_cost = |n: usize| {
        u64::try_from(n)
            .ok()?
            .checked_mul(log2_ceil(n).saturating_add(1))
    };
    sort_cost(entities)?
        .checked_add(sort_cost(aspects)?)?
        .checked_add(sort_cost(adjacencies)?)?
        .checked_add(aspect_bytes)?
        .checked_add(adj_bytes)
}

#[allow(clippy::too_many_arguments)]
fn claim_scratch_memory(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    allocation: &SourceFactsAllocation,
    entities: usize,
    aspects: usize,
    adjacencies: usize,
    aspect_bytes: u64,
    adj_bytes: u64,
    subject: &str,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let stop = |cause| relational_read_denial(cause, subject);
    allocation
        .claim_slice::<EntityId>(execution, entities)
        .map_err(stop)?;
    allocation
        .claim_slice::<(bool, usize, WorthQueryObservedFieldRevision)>(execution, aspects)
        .map_err(stop)?;
    allocation
        .claim_slice::<(bool, usize, WorthQueryObservedAdjacencyRevision)>(execution, adjacencies)
        .map_err(stop)?;
    let extra = aspect_bytes
        .checked_add(adj_bytes)
        .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    if extra > 0 {
        allocation.claim(execution, extra).map_err(stop)?;
    }
    Ok(())
}

fn exact_copy_entities(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    allocation: &SourceFactsAllocation,
    deduped: &mut Vec<EntityId>,
    subject: &str,
) -> Result<Vec<EntityId>, WorthQueryApplicationReadExecutionDenial> {
    let stop = |cause| relational_read_denial(cause, subject);
    let len = deduped.len();
    allocation
        .claim_slice::<EntityId>(execution, len)
        .map_err(stop)?;
    let mut out = try_vec_exact(len)?;
    out.extend_from_slice(deduped);
    debug_assert_eq!(out.capacity(), len);
    deduped.clear();
    Ok(out)
}

fn drain_exact<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    allocation: &SourceFactsAllocation,
    tagged: &mut Vec<(bool, usize, T)>,
    payload_bytes: u64,
    subject: &str,
) -> Result<Vec<T>, WorthQueryApplicationReadExecutionDenial> {
    let stop = |cause| relational_read_denial(cause, subject);
    let len = tagged.len();
    allocation.claim_slice::<T>(execution, len).map_err(stop)?;
    if payload_bytes > 0 {
        allocation.claim(execution, payload_bytes).map_err(stop)?;
    }
    let mut out = try_vec_exact(len)?;
    for (_, _, item) in tagged.drain(..) {
        out.push(item);
    }
    debug_assert_eq!(out.capacity(), len);
    Ok(out)
}

fn merge_entities(
    base: &[EntityId],
    narrowing: &[EntityId],
    total: usize,
) -> Result<Vec<EntityId>, WorthQueryApplicationReadExecutionDenial> {
    let mut out = try_vec_exact(total)?;
    out.extend_from_slice(base);
    out.extend_from_slice(narrowing);
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

fn merge_tagged_aspects(
    base: &[WorthQueryObservedFieldRevision],
    narrowing: &[WorthQueryObservedFieldRevision],
    total: usize,
) -> Result<
    Vec<(bool, usize, WorthQueryObservedFieldRevision)>,
    WorthQueryApplicationReadExecutionDenial,
> {
    let mut out = try_vec_exact(total)?;
    for (idx, item) in base.iter().enumerate() {
        out.push((false, idx, item.clone()));
    }
    for (idx, item) in narrowing.iter().enumerate() {
        out.push((true, idx, item.clone()));
    }
    out.sort_unstable_by(|a, b| {
        (a.2.entity, &a.2.aspect, &a.2.field, a.0, a.1).cmp(&(
            b.2.entity,
            &b.2.aspect,
            &b.2.field,
            b.0,
            b.1,
        ))
    });
    out.dedup_by(|later, earlier| {
        later.2.entity == earlier.2.entity
            && later.2.aspect == earlier.2.aspect
            && later.2.field == earlier.2.field
    });
    Ok(out)
}

fn merge_tagged_adjacencies(
    base: &[WorthQueryObservedAdjacencyRevision],
    narrowing: &[WorthQueryObservedAdjacencyRevision],
    total: usize,
) -> Result<
    Vec<(bool, usize, WorthQueryObservedAdjacencyRevision)>,
    WorthQueryApplicationReadExecutionDenial,
> {
    let mut out = try_vec_exact(total)?;
    for (idx, item) in base.iter().enumerate() {
        out.push((false, idx, item.clone()));
    }
    for (idx, item) in narrowing.iter().enumerate() {
        out.push((true, idx, item.clone()));
    }
    out.sort_unstable_by(|a, b| {
        (
            a.2.anchor,
            a.2.relation_kind,
            direction_rank(a.2.direction),
            a.0,
            a.1,
        )
            .cmp(&(
                b.2.anchor,
                b.2.relation_kind,
                direction_rank(b.2.direction),
                b.0,
                b.1,
            ))
    });
    out.dedup_by(|later, earlier| {
        later.2.anchor == earlier.2.anchor
            && later.2.relation_kind == earlier.2.relation_kind
            && later.2.direction == earlier.2.direction
    });
    Ok(out)
}

const fn direction_rank(direction: RelationalAdjacencyDirection) -> u8 {
    match direction {
        RelationalAdjacencyDirection::Outgoing => 0,
        RelationalAdjacencyDirection::Incoming => 1,
    }
}
