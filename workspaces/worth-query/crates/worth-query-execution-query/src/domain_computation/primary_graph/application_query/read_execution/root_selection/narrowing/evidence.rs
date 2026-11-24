//! Narrowing observations join the selection's own result-set evidence.
//!
//! Every merge scratch buffer, cloned field-revision/adjacency payload, and
//! O(n log n) sort/dedup step is preflighted and charged through
//! `SourceFactsAllocation` before allocating or cloning.
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
/// deduplicated in canonical key order. A caller that captured no set
/// evidence keeps none.
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
    // Declaration admits narrowing only over exact-root and path selections.
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
    claim_merge_memory(
        execution,
        &scratch,
        total_entities,
        total_aspects,
        total_adjacencies,
        aspect_clone_bytes,
        adj_clone_bytes,
        subject,
    )?;

    let entities = merge_entities(&base.entities, &narrowing.entities, total_entities)?;
    let aspects = merge_aspects(&base.aspects, &narrowing.aspects, total_aspects)?;
    let adjacencies =
        merge_adjacencies(&base.adjacencies, &narrowing.adjacencies, total_adjacencies)?;

    if let Some(allocation) = &base.allocation {
        let kept_aspect_bytes =
            field_revision_bytes(&aspects, &[]).ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
        let kept_adj_bytes = adjacency_revision_bytes(&adjacencies, &[])
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
        claim_merge_memory(
            execution,
            allocation,
            entities.len(),
            aspects.len(),
            adjacencies.len(),
            kept_aspect_bytes,
            kept_adj_bytes,
            subject,
        )?;
    }
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
    let mut bytes = 0_u64;
    for rev in left.iter().chain(right) {
        bytes = bytes.checked_add(u64::try_from(rev.owned_capacity_bytes()).ok()?)?;
    }
    Some(bytes)
}

fn adjacency_revision_bytes(
    left: &[WorthQueryObservedAdjacencyRevision],
    right: &[WorthQueryObservedAdjacencyRevision],
) -> Option<u64> {
    let mut bytes = 0_u64;
    for rev in left.iter().chain(right) {
        bytes = bytes.checked_add(u64::try_from(rev.owned_capacity_bytes()).ok()?)?;
    }
    Some(bytes)
}

fn merge_work_units(
    entities: usize,
    aspects: usize,
    adjacencies: usize,
    aspect_bytes: u64,
    adj_bytes: u64,
) -> Option<u64> {
    let sort_cost = |n: usize| u64::try_from(n).ok()?.checked_mul(log2_ceil(n));
    sort_cost(entities)?
        .checked_add(sort_cost(aspects)?)?
        .checked_add(sort_cost(adjacencies)?)?
        .checked_add(aspect_bytes)?
        .checked_add(adj_bytes)
}

#[allow(clippy::too_many_arguments)]
fn claim_merge_memory(
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
        .claim_slice::<WorthQueryObservedFieldRevision>(execution, aspects)
        .map_err(stop)?;
    allocation
        .claim_slice::<WorthQueryObservedAdjacencyRevision>(execution, adjacencies)
        .map_err(stop)?;
    let extra = aspect_bytes
        .checked_add(adj_bytes)
        .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
    if extra > 0 {
        allocation.claim(execution, extra).map_err(stop)?;
    }
    Ok(())
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

fn merge_aspects(
    base: &[WorthQueryObservedFieldRevision],
    narrowing: &[WorthQueryObservedFieldRevision],
    total: usize,
) -> Result<Vec<WorthQueryObservedFieldRevision>, WorthQueryApplicationReadExecutionDenial> {
    let mut out: Vec<WorthQueryObservedFieldRevision> = try_vec_exact(total)?;
    out.extend(base.iter().cloned());
    out.extend(narrowing.iter().cloned());
    out.sort_by(|a, b| (a.entity, &a.aspect, &a.field).cmp(&(b.entity, &b.aspect, &b.field)));
    out.dedup_by(|later, earlier| {
        later.entity == earlier.entity
            && later.aspect == earlier.aspect
            && later.field == earlier.field
    });
    Ok(out)
}

fn merge_adjacencies(
    base: &[WorthQueryObservedAdjacencyRevision],
    narrowing: &[WorthQueryObservedAdjacencyRevision],
    total: usize,
) -> Result<Vec<WorthQueryObservedAdjacencyRevision>, WorthQueryApplicationReadExecutionDenial> {
    let mut out: Vec<WorthQueryObservedAdjacencyRevision> = try_vec_exact(total)?;
    out.extend(base.iter().cloned());
    out.extend(narrowing.iter().cloned());
    out.sort_by(|a, b| {
        (a.anchor, a.relation_kind, direction_rank(a.direction)).cmp(&(
            b.anchor,
            b.relation_kind,
            direction_rank(b.direction),
        ))
    });
    out.dedup_by(|later, earlier| {
        later.anchor == earlier.anchor
            && later.relation_kind == earlier.relation_kind
            && later.direction == earlier.direction
    });
    Ok(out)
}

const fn direction_rank(direction: RelationalAdjacencyDirection) -> u8 {
    match direction {
        RelationalAdjacencyDirection::Outgoing => 0,
        RelationalAdjacencyDirection::Incoming => 1,
    }
}
