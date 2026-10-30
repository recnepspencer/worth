//! Query admission, controls and receipt materialization around the sole path engine.
use super::{BoundedRootSelection, RootSelectionWork};
use crate::domain_computation::primary_graph::{
    application_query::{
        observed_source::WorthQueryObservedRootSelection,
        read_execution::{
            read_execution_denial, read_plan::ReadPlan, OneShotReadWorkObservation,
            WorthQueryApplicationReadExecutionDenial, WorthQueryApplicationReadExecutionDenialKind,
        },
    },
    root_path_selection::{
        self, RootPathOrientation, RootPathReadBasis, RootPathReadPorts, RootPathSelection,
        RootPathSelectionRequest,
    },
    source_facts::{SourceFacts, SourceFactsAllocation},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use worth_query_installation::facade::WorthQueryInstalledRootPath;
use worth_relational::facade::identity::EntityId;
mod adapter;
pub(super) fn prepare_steps(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    paths: &[WorthQueryInstalledRootPath],
    work: &mut RootSelectionWork<'_>,
    subject: &str,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let mut count = 0usize;
    for path in paths {
        work.checkpoint(subject)?;
        count = count
            .checked_add(path.steps().len())
            .ok_or_else(|| traversal_denial(subject))?;
    }
    work.reserve_steps(execution, count, subject)
}
pub(super) fn select_paths(
    basis: &RootPathReadBasis<'_, '_, '_>,
    plan: &ReadPlan<'_>,
    work: &mut RootSelectionWork<'_>,
    request: RootPathSelectionRequest<'_>,
) -> Result<RootPathSelection, WorthQueryApplicationReadExecutionDenial> {
    let interruption = work.interruption;
    let mut observe = |event: root_path_selection::RootPathReadObservation<'_>| {
        adapter::observe(work, basis.execution, event)
    };
    let checkpoint = |subject: &str| interruption.checkpoint(subject);
    let mut guard = |guard: &worth_query_installation::facade::WorthQueryInstalledRootPathGuard,
                     candidates: &BTreeSet<EntityId>,
                     prepare: &mut dyn FnMut() -> Result<
        (),
        root_path_selection::RootPathReadDenial<WorthQueryApplicationReadExecutionDenial>,
    >| {
        adapter::read_guard(
            basis.execution,
            basis.runtime,
            basis.graph,
            plan,
            guard,
            candidates,
            prepare,
        )
    };
    root_path_selection::select(
        basis,
        request,
        &mut RootPathReadPorts {
            checkpoint: &checkpoint,
            read_guard_with_sources: &mut guard,
            observation: &mut observe,
        },
    )
    .map_err(adapter::denial)
}
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
    prepare_steps(execution, paths, &mut work, plan.name)?;
    let result = select_paths(
        &RootPathReadBasis {
            execution,
            runtime,
            graph,
            projection,
        },
        plan,
        &mut work,
        RootPathSelectionRequest {
            allocation: plan.material_source,
            paths,
            start: plan.root,
            orientation: RootPathOrientation::ForwardCensus,
            capture_set: capture_result_set,
            subject: plan.name,
        },
    )?;
    let candidates = collect_candidates(
        execution,
        result.allocation.as_deref(),
        result.roots.keys().copied(),
    )?;
    let result_set_source = result
        .set_source
        .map(|source| finish_source(execution, source))
        .transpose()?;
    let root_path_source = if result.roots.is_empty() {
        None
    } else {
        if let Some(allocation) = &result.allocation {
            allocation
                .claim_tree_copy::<(EntityId, Arc<WorthQueryObservedRootSelection>)>(
                    execution,
                    result.roots.len(),
                )
                .map_err(|cause| {
                    super::super::denial::relational_read_denial(cause, "source witness map")
                })?;
        }
        let mut sources = BTreeMap::new();
        for (root, source) in result.roots {
            sources.insert(root, finish_source(execution, source)?);
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
    })
}
fn collect_candidates(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    allocation: Option<&SourceFactsAllocation>,
    values: impl ExactSizeIterator<Item = EntityId>,
) -> Result<Vec<EntityId>, WorthQueryApplicationReadExecutionDenial> {
    if let Some(allocation) = allocation {
        allocation
            .claim_slice::<EntityId>(execution, values.len())
            .map_err(|cause| {
                super::super::denial::relational_read_denial(cause, "path candidates")
            })?;
    }
    let mut result = Vec::new();
    result.try_reserve_exact(values.len()).map_err(|_| super::super::denial::relational_read_denial(worth_relational::facade::transactions::RelationalExecutionDenialCause::AllocationFailed, "path candidates"))?;
    result.extend(values);
    Ok(result)
}
fn finish_source(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    facts: SourceFacts,
) -> Result<Arc<WorthQueryObservedRootSelection>, WorthQueryApplicationReadExecutionDenial> {
    facts
        .admit_container::<WorthQueryObservedRootSelection>(execution)
        .map_err(|cause| {
            super::super::denial::relational_read_denial(cause, "source witness allocation")
        })?;
    Ok(Arc::new(WorthQueryObservedRootSelection::new(facts, None)))
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
