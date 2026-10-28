use super::path_union::*;
use super::*;
use crate::domain_computation::primary_graph::application_query::read_execution::read_plan::ReadPlan;
use std::collections::{BTreeMap, BTreeSet};
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_query_installation::facade::WorthQueryInstalledRootPath;
use worth_relational::facade::identity::EntityId;

pub(super) fn apply_guards(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    path: &WorthQueryInstalledRootPath,
    after_step: usize,
    frontier: &mut BTreeMap<EntityId, RootPathSourceBuilder>,
    work: &mut RootSelectionWork,
    set_source: &mut Option<RootPathSourceBuilder>,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    for guard in path
        .guards()
        .iter()
        .filter(|guard| guard.after_step() == after_step)
    {
        if frontier.is_empty() {
            break;
        }
        let layout = graph
            .equality_field(
                guard.entity(),
                guard.aspect().as_str(),
                guard.field().as_str(),
            )
            .ok_or_else(|| traversal_denial(guard.field().as_str()))?;
        let computation = plan
            .governance
            .admit_internal_projection(
                (
                    guard.entity(),
                    guard.aspect().as_str(),
                    guard.field().as_str(),
                ),
                guard.field(),
                ApplicationQueryObservableInfluence::RowPresence,
            )
            .ok_or_else(|| traversal_denial(guard.field().as_str()))?;
        if !computation.admits_locator(&layout.locator) {
            return Err(traversal_denial(guard.field().as_str()));
        }
        for (&entity, source) in &mut *frontier {
            let observed = source.observe_guard_field(
                execution,
                projection,
                graph,
                entity,
                guard.entity(),
                guard.aspect(),
                guard.field(),
                work,
            )?;
            if let Some(set_source) = &mut *set_source
                && set_source.record_field(execution, observed)?
            {
                work.charge_source_copy(1, guard.field().as_str())?;
            }
        }
        let candidates = frontier.keys().copied().collect::<BTreeSet<_>>();
        work.checkpoint(guard.field().as_str())?;
        let read = read_governed_root_guard(
            execution,
            runtime,
            computation,
            &candidates,
            layout.entity_kind,
            &layout.locator,
            guard.expected(),
        )
        .map_err(|stop| frontier_denial(stop, guard.field().as_str()))?;
        work.charge_predicate(
            read.entity_records_examined(),
            read.matching_entity_ids_reserved(),
            read.field_reads(),
            guard.field().as_str(),
        )?;
        let matching = read.into_matching_entity_ids();
        frontier.retain(|entity, _| matching.contains(entity));
    }
    Ok(())
}
