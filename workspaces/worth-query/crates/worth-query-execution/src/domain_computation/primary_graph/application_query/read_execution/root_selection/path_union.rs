use std::collections::{BTreeMap, BTreeSet};

use super::guards::apply_guards;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryRootPathTraversal, RelationDirection,
};
use worth_query_installation::facade::WorthQueryInstalledRootPath;
use worth_relational::facade::identity::EntityId;

use super::{evidence::RootPathSourceBuilder, BoundedRootSelection, RootSelectionWork};
use crate::domain_computation::primary_graph::application_query::{
    read_execution::read_plan::ReadPlan,
    read_execution::{
        read_execution_denial, OneShotReadWorkObservation,
        WorthQueryApplicationReadExecutionDenial, WorthQueryApplicationReadExecutionDenialKind,
    },
};

pub(super) fn select_root_path_union(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    paths: &[WorthQueryInstalledRootPath],

    capture_result_set: bool,
    spent: Option<&OneShotReadWorkObservation<'_>>,

    interruption: super::super::ReadInterruption<'_>,
) -> Result<BoundedRootSelection, WorthQueryApplicationReadExecutionDenial> {
    let projection = runtime
        .project_snapshot(plan.snapshot)
        .ok_or_else(|| traversal_denial(plan.name))?;
    let mut work = RootSelectionWork::new(interruption, spent);
    let mut step_count = 0usize;
    for path in paths {
        work.checkpoint(plan.name)?;
        step_count = step_count
            .checked_add(path.steps().len())
            .ok_or_else(|| traversal_denial(plan.name))?;
    }
    work.reserve_steps(execution, step_count, plan.name)?;
    let mut roots = BTreeMap::new();
    let mut set_source = capture_result_set.then(RootPathSourceBuilder::default);
    if let Some(source) = &mut set_source {
        source.observe_entity(execution, plan.root)?;
    }
    for (path_index, path) in paths.iter().enumerate() {
        let terminal = traverse_path(
            execution,
            runtime,
            &projection,
            graph,
            plan,
            path,
            path_index,
            &mut work,
            &mut set_source,
        )?;
        for (root, source) in terminal {
            if !roots.contains_key(&root) {
                work.admit_root_entry::<RootPathSourceBuilder>(execution, roots.len())?;
            }
            roots.entry(root).or_insert(source);
        }
    }
    if let Some(set_source) = &mut set_source {
        for source in roots.values() {
            for entity in source.entities() {
                if set_source.observe_entity(execution, entity)? {
                    work.charge_source_copy(1, plan.name)?;
                }
            }
        }
    }
    work.admit_candidates(execution, roots.len())?;
    let candidates = roots.keys().copied().collect();
    let result_set_source = set_source
        .map(|source| source.finish(execution))
        .transpose()?;
    let root_path_source = if roots.is_empty() {
        None
    } else {
        let mut sources = BTreeMap::new();
        for (root, mut source) in roots {
            if let Some(allocation) = &work.source_allocation {
                source.protect(execution, allocation)?;
            }
            work.admit_root_entry::<std::sync::Arc<crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedRootSelection>>(execution, sources.len())?;
            sources.insert(root, source.finish(execution)?);
        }
        Some(sources)
    };
    Ok(BoundedRootSelection {
        candidates,
        selected_predicate_source: None,
        root_path_source,
        result_set_source,
        examined_candidates: work.predicate_records_examined,
        predicate_work_units: work.predicate_work_units,
        work_units: work.work_units,
        predicate_index_generation: None,
        reads: work.reads,
        root_path_steps: work.take_steps(),
        _source_allocation: work.source_allocation.clone(),
    })
}

fn traverse_path(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    path: &WorthQueryInstalledRootPath,
    path_index: usize,
    work: &mut RootSelectionWork,
    set_source: &mut Option<RootPathSourceBuilder>,
) -> Result<BTreeMap<EntityId, RootPathSourceBuilder>, WorthQueryApplicationReadExecutionDenial> {
    let mut seed = RootPathSourceBuilder::default();
    if let Some(allocation) = &work.source_allocation {
        seed.protect(execution, allocation)?;
    }
    seed.observe_entity(execution, plan.root)?;
    let mut frontier = BTreeMap::new();
    work.admit_root_entry::<RootPathSourceBuilder>(execution, 0)?;
    frontier.insert(plan.root, seed);
    apply_guards(
        execution,
        runtime,
        projection,
        graph,
        plan,
        path,
        0,
        &mut frontier,
        work,
        set_source,
    )?;
    for (step_index, step) in path.steps().iter().enumerate() {
        if frontier.is_empty() {
            work.record_step(
                execution,
                path_index,
                step_index,
                step.traversal(),
                step.direction(),
                Default::default(),
                step.relation(),
            )?;
            continue;
        }
        let layout = graph
            .relation(step.relation())
            .filter(|layout| {
                graph.entity_kind(step.from()) == Some(layout.from)
                    && graph.entity_kind(step.to()) == Some(layout.to)
            })
            .ok_or_else(|| traversal_denial(step.relation()))?;
        let next = if step.traversal() == ApplicationQueryRootPathTraversal::Closure {
            super::closure::advance(
                execution, runtime, layout, step, &frontier, work, set_source, path_index,
                step_index,
            )?
        } else {
            let anchors = frontier.keys().copied().collect::<BTreeSet<_>>();
            let direction = match step.direction() {
                RelationDirection::Forward => {
                    worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing
                }
                RelationDirection::Reverse => {
                    worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming
                }
            };
            for (&anchor, source) in &mut frontier {
                let observed = source.observe_adjacencies(
                    execution,
                    projection,
                    anchor,
                    layout.kind,
                    direction,
                    step.relation(),
                    work,
                )?;
                if let Some(set_source) = set_source
                    && set_source.record_adjacency(execution, observed)?
                {
                    work.charge_source_copy(1, step.relation())?;
                }
            }
            work.checkpoint(step.relation())?;
            let read = match step.direction() {
                RelationDirection::Forward => runtime.bounded_outgoing_relations_for_frontier(
                    execution,
                    &anchors,
                    layout.kind,
                    None,
                ),
                RelationDirection::Reverse => runtime.bounded_incoming_relations_for_frontier(
                    execution,
                    &anchors,
                    layout.kind,
                    None,
                ),
            }
            .map_err(|stop| frontier_denial(stop, step.relation()))?;
            work.charge(
                read.adjacency_lists_read(),
                read.relation_records_examined(),
                read.endpoint_records_reserved(),
                step.relation(),
            )?;
            let mut step_reads = crate::domain_computation::primary_graph::read_observation::WorthQueryInvariantProjectionWork::default();
            step_reads.record_adjacencies(
                read.adjacency_lists_read(),
                read.relation_records_examined(),
            );
            work.record_step(
                execution,
                path_index,
                step_index,
                step.traversal(),
                step.direction(),
                step_reads,
                step.relation(),
            )?;
            let mut next = BTreeMap::new();
            for record in read.into_records() {
                work.checkpoint(step.relation())?;
                let (anchor, endpoint) = match step.direction() {
                    RelationDirection::Forward => (record.source, record.target),
                    RelationDirection::Reverse => (record.target, record.source),
                };
                if next.contains_key(&endpoint) {
                    continue;
                }
                let predecessor = frontier
                    .get(&anchor)
                    .ok_or_else(|| traversal_denial(step.relation()))?;
                work.charge_source_copy(predecessor.copy_cost(), step.relation())?;
                work.admit_root_entry::<RootPathSourceBuilder>(execution, next.len())?;
                let mut source =
                    predecessor.copy_for_step(execution, work.source_allocation.as_ref())?;
                source.observe_entity(execution, endpoint)?;
                next.insert(endpoint, source);
            }
            next
        };
        frontier = next;
        apply_guards(
            execution,
            runtime,
            projection,
            graph,
            plan,
            path,
            step_index.saturating_add(1),
            &mut frontier,
            work,
            set_source,
        )?;
    }
    Ok(frontier)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn read_governed_root_guard(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    _projection: crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationInternalProjectionAdmission<'_>,
    frontier: &BTreeSet<EntityId>,
    entity_kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    expected: &worth_foundational::facade::AspectValue,
) -> Result<
    worth_relational::facade::runtime::BoundedFrontierFieldEqualityTruthRead,
    worth_relational::facade::runtime::BoundedFrontierReadStop<
        worth_relational::facade::runtime::FrontierFieldEqualityTruthReadLimitExceeded,
        worth_execution::WorkCeilingDenial,
    >,
> {
    runtime.bounded_entity_field_equals_for_frontier(
        execution,
        frontier,
        entity_kind,
        locator,
        expected,
        None,
    )
}

pub(super) fn traversal_denial(subject: &str) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::TraversalUnavailable,
        subject,
    )
}

fn work_limit_denial(subject: &str) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
        subject,
    )
}

pub(super) fn frontier_denial<L>(
    stop: worth_relational::facade::runtime::BoundedFrontierReadStop<
        L,
        worth_execution::WorkCeilingDenial,
    >,
    subject: &str,
) -> WorthQueryApplicationReadExecutionDenial {
    match stop {
        worth_relational::facade::runtime::BoundedFrontierReadStop::Limit(_) => {
            work_limit_denial(subject)
        }
        worth_relational::facade::runtime::BoundedFrontierReadStop::Admission(stop) => {
            super::super::denial::relational_read_denial(
                worth_relational::facade::transactions::RelationalExecutionDenialCause::from(stop),
                subject,
            )
        }
    }
}
