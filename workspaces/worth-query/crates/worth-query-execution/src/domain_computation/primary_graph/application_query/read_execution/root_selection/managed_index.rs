//! Consume only a sealed provider selection; legacy scope selection is never a fallback.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{
        observed_source::DeclaredManagedIndexSelection,
        one_shot::outcome::material::MaterialCompletion,
    },
    source_facts::SourceFactsAllocation,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use worth_relational::facade::{
    runtime::ProjectionAspectScope, storage::RecordLifecycleState,
    transactions::RelationalExecutionDenialCause as Cause,
};
mod predicates;
use super::super::denial::relational_read_denial;

#[allow(clippy::too_many_arguments)]
pub(super) fn select(
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    capture_result_set: bool,
    spent: Option<&OneShotReadWorkObservation<'_>>,
    interruption: super::super::ReadInterruption<'_>,
) -> Result<BoundedRootSelection, WorthQueryApplicationReadExecutionDenial> {
    let refused = || path_union::traversal_denial(plan.name);
    let rows = plan.managed_rows.ok_or_else(refused)?;
    if !rows.admits(plan.parent_session, plan.binding, plan.root) {
        return Err(refused());
    }
    let allocation = plan.material_source.ok_or_else(refused)?;
    let storage = MaterialCompletion::new(execution, allocation);
    let selected = plan
        .contract
        .managed_index_selection()
        .ok_or_else(refused)?;
    let actual_name = &rows.association().definition().parts().name;
    let work = actual_name
        .len()
        .checked_add(selected.parts().index.len())
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| relational_read_denial(Cause::WorkCounterOverflow, plan.name))?;
    storage
        .work(work)
        .map_err(|cause| relational_read_denial(cause, plan.name))?;
    if actual_name != &selected.parts().index {
        return Err(refused());
    }
    let projection = runtime
        .project_snapshot(plan.snapshot)
        .ok_or_else(refused)?;
    let bound = graph
        .application_entity_kinds()
        .size_hint()
        .1
        .ok_or_else(refused)?;
    let work = plan
        .contract
        .root_entity()
        .len()
        .checked_add(1)
        .and_then(|n| n.checked_mul(bound.checked_add(1)?))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| relational_read_denial(Cause::WorkCounterOverflow, plan.name))?;
    storage
        .work(work)
        .map_err(|cause| relational_read_denial(cause, plan.name))?;
    let kind = graph
        .entity_kind(plan.contract.root_entity())
        .ok_or_else(refused)?;
    let mut work = RootSelectionWork::new(interruption, spent);
    let mut source = RootPathSourceBuilder::for_material_source(execution, Some(allocation))?;
    source.set_managed_index(DeclaredManagedIndexSelection {
        access: Arc::clone(rows.association().access_contract()),
        dependencies: Arc::clone(rows.dependencies()),
    });
    source.observe_entity(execution, plan.root)?;
    let mut frontier = BTreeSet::new();
    for root in rows.roots() {
        work.checkpoint(plan.name)?;
        work.charge(0, 0, 1, plan.name)?;
        projection
            .entity_record_with_projection_scope(
                execution,
                *root,
                ProjectionAspectScope::empty(),
                |record| {
                    work.reads.record_endpoint();
                    Some(
                        record.kind_id() == kind
                            && record.lifecycle() == RecordLifecycleState::Live,
                    )
                },
            )
            .map_err(|cause| relational_read_denial(cause, plan.name))?
            .filter(|valid| *valid)
            .ok_or_else(refused)?;
        source.observe_entity(execution, *root)?;
        insert(execution, allocation, &mut frontier, *root, plan.name)?;
    }
    let examined_candidates = frontier.len();
    let frontier = predicates::filter(
        execution,
        runtime,
        graph,
        plan,
        &projection,
        frontier,
        &mut source,
        &mut work,
        allocation,
    )?;
    // The original proof was checked immediately before Native borrowing;
    // publication/cleanup may still invalidate its exact retained epoch.
    if !rows.admits(plan.parent_session, plan.binding, plan.root) {
        return Err(refused());
    }
    let source = source.finish(execution)?;
    allocation
        .claim_slice::<EntityId>(execution, frontier.len())
        .map_err(|cause| relational_read_denial(cause, plan.name))?;
    allocation
        .claim_tree_copy::<(
            EntityId,
            Arc<super::super::super::observed_source::WorthQueryObservedRootSelection>,
        )>(execution, frontier.len())
        .map_err(|cause| relational_read_denial(cause, plan.name))?;
    let mut candidates = Vec::new();
    candidates
        .try_reserve_exact(frontier.len())
        .map_err(|_| relational_read_denial(Cause::AllocationFailed, plan.name))?;
    let mut sources = BTreeMap::new();
    for root in frontier {
        admit_search(&storage, sources.len(), plan.name)?;
        candidates.push(root);
        sources.insert(root, Arc::clone(&source));
    }
    Ok(BoundedRootSelection {
        candidates,
        selected_predicate_source: None,
        root_path_source: (!sources.is_empty()).then_some(sources),
        result_set_source: capture_result_set.then_some(source),
        examined_candidates,
        predicate_work_units: work.predicate_work_units,
        work_units: work.work_units,
        predicate_index_generation: None,
        reads: work.reads,
        root_path_steps: work.take_steps(),
    })
}

fn admit_search(
    storage: &MaterialCompletion<'_, '_, '_>,
    count: usize,
    subject: &str,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let units = count
        .checked_add(1)
        .and_then(|n| n.checked_mul(size_of::<EntityId>()))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| relational_read_denial(Cause::WorkCounterOverflow, subject))?;
    storage
        .work(units)
        .map_err(|cause| relational_read_denial(cause, subject))
}
fn insert(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    allocation: &Arc<SourceFactsAllocation>,
    values: &mut BTreeSet<EntityId>,
    value: EntityId,
    subject: &str,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    admit_search(
        &MaterialCompletion::new(execution, allocation),
        values.len(),
        subject,
    )?;
    allocation
        .claim_tree::<EntityId>(execution, values.len())
        .map_err(|cause| relational_read_denial(cause, subject))?;
    values.insert(value);
    Ok(())
}
