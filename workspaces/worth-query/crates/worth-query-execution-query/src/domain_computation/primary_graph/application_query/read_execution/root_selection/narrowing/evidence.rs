//! Narrowing observations join the selection's own result-set evidence.
use super::RootPathSourceBuilder;
use crate::domain_computation::primary_graph::application_query::{
    observed_source::{WorthQueryObservedFieldRevision, WorthQueryObservedRootSelection},
    read_execution::{
        relational_read_denial, root_selection::path_union::traversal_denial,
        WorthQueryApplicationReadExecutionDenial,
    },
};
use crate::domain_computation::primary_graph::source_facts::SourceFacts;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// One canonical set witness: the selector's reads plus every narrowing
/// read, deduplicated in canonical key order. A caller that captured no
/// set evidence keeps none.
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
    let stop = |cause| relational_read_denial(cause, "narrowed selection evidence");
    if let Some(allocation) = &base.allocation {
        allocation
            .claim_slice::<worth_relational::facade::identity::EntityId>(execution, entities.len())
            .map_err(stop)?;
        allocation
            .claim_slice::<WorthQueryObservedFieldRevision>(execution, aspects.len())
            .map_err(stop)?;
    }
    let facts = SourceFacts {
        entities: entities.into_iter().collect(),
        aspects: aspects.into_values().collect(),
        adjacencies: base.adjacencies.clone(),
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
