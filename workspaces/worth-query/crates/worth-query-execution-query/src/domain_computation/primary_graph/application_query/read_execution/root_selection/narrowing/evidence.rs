//! Narrowing observations join the selection's own result-set evidence.
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
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationalAdjacencyDirection,
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
) -> Result<Option<Arc<WorthQueryObservedRootSelection>>, WorthQueryApplicationReadExecutionDenial>
{
    let Some(base) = base else {
        return Ok(None);
    };
    // Declaration admits narrowing only over exact-root and path selections.
    if base.orbit.is_some() || base.components.is_some() || base.managed_index.is_some() {
        return Err(traversal_denial("narrowed selection evidence"));
    }
    let narrowing = narrowing.finish(execution)?;
    let stop = |cause| relational_read_denial(cause, "narrowed selection evidence");
    let entities: BTreeSet<_> = base
        .entities
        .iter()
        .chain(&narrowing.entities)
        .copied()
        .collect();
    let mut aspects = BTreeMap::new();
    for revision in base.aspects.iter().chain(&narrowing.aspects) {
        aspects
            .entry((
                revision.entity,
                revision.aspect.clone(),
                revision.field.clone(),
            ))
            .or_insert_with(|| revision.clone());
    }
    let adjacencies = if narrowing.adjacencies.is_empty() {
        base.adjacencies.clone()
    } else {
        let total_adjacencies = base
            .adjacencies
            .len()
            .checked_add(narrowing.adjacencies.len())
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        let adj_merge_work = super::residual::log2_ceil(total_adjacencies)
            .checked_mul(total_adjacencies as u64)
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        if adj_merge_work > 0 {
            SourceFactsAllocation::work(execution, adj_merge_work).map_err(stop)?;
        }
        let scratch = SourceFactsAllocation::open(execution).map_err(stop)?;
        scratch
            .claim_tree_copy::<((EntityId, KindId, u8), WorthQueryObservedAdjacencyRevision)>(
                execution,
                total_adjacencies,
            )
            .map_err(stop)?;
        let mut merged = BTreeMap::new();
        for revision in base.adjacencies.iter().chain(&narrowing.adjacencies) {
            merged
                .entry((
                    revision.anchor,
                    revision.relation_kind,
                    direction_rank(revision.direction),
                ))
                .or_insert_with(|| revision.clone());
        }
        if let Some(allocation) = &base.allocation {
            allocation
                .claim_slice::<WorthQueryObservedAdjacencyRevision>(execution, merged.len())
                .map_err(stop)?;
        }
        collect_exact(merged.into_values()).map_err(stop)?
    };
    if let Some(allocation) = &base.allocation {
        allocation
            .claim_slice::<EntityId>(execution, entities.len())
            .map_err(stop)?;
        allocation
            .claim_slice::<WorthQueryObservedFieldRevision>(execution, aspects.len())
            .map_err(stop)?;
    }
    let facts = SourceFacts {
        entities: collect_exact(entities.into_iter()).map_err(stop)?,
        aspects: collect_exact(aspects.into_values()).map_err(stop)?,
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

const fn direction_rank(direction: RelationalAdjacencyDirection) -> u8 {
    match direction {
        RelationalAdjacencyDirection::Outgoing => 0,
        RelationalAdjacencyDirection::Incoming => 1,
    }
}

fn collect_exact<T>(values: impl ExactSizeIterator<Item = T>) -> Result<Vec<T>, Cause> {
    let mut out = Vec::new();
    out.try_reserve_exact(values.len())
        .map_err(|_| Cause::AllocationFailed)?;
    out.extend(values);
    Ok(out)
}
