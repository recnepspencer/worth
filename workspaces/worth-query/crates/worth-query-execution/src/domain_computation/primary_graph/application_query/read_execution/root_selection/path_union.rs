use std::collections::BTreeSet;

use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_query_declaration::facade::application_query::ApplicationQueryRootPathDirection;
use worth_query_installation::facade::WorthQueryInstalledRootPath;
use worth_relational::facade::identity::EntityId;

use super::{BoundedRootSelection, RootSelectionWork};
use crate::domain_computation::primary_graph::application_query::{
    read_execution::{
        read_execution_denial, WorthQueryApplicationReadExecutionDenial,
        WorthQueryApplicationReadExecutionDenialKind,
    },
    WorthQueryAdmittedApplicationQueryPlan,
};

pub(super) fn select_root_path_union<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    paths: &[WorthQueryInstalledRootPath],
) -> Result<BoundedRootSelection, WorthQueryApplicationReadExecutionDenial> {
    let mut roots = BTreeSet::new();
    let mut work = RootSelectionWork::new(plan.controls.maximum_work().get());
    for path in paths {
        let terminal = traverse_path(runtime, graph, plan, path, &mut work)?;
        roots.extend(terminal);
    }
    Ok(BoundedRootSelection {
        candidates: roots.into_iter().collect(),
        selected_predicate_source: None,
        examined_candidates: work.predicate_records_examined,
        predicate_work_units: work.predicate_work_units,
        work_units: work.work_units,
        predicate_index_generation: None,
        adjacency_lists_read: work.adjacency_lists_read,
        relation_records_examined: work.relation_records_examined,
    })
}

fn traverse_path<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    path: &WorthQueryInstalledRootPath,
    work: &mut RootSelectionWork,
) -> Result<BTreeSet<EntityId>, WorthQueryApplicationReadExecutionDenial> {
    let mut frontier = BTreeSet::from([plan.scope.entity_id()]);
    apply_guards(runtime, graph, plan, path, 0, &mut frontier, work)?;
    for (step_index, step) in path.steps().iter().enumerate() {
        if frontier.is_empty() {
            break;
        }
        let layout = graph
            .relation(step.relation())
            .filter(|layout| {
                graph.entity_kind(step.from()) == Some(layout.from)
                    && graph.entity_kind(step.to()) == Some(layout.to)
            })
            .ok_or_else(|| traversal_denial(step.relation()))?;
        let read = match step.direction() {
            ApplicationQueryRootPathDirection::Forward => runtime
                .read_truth()
                .bounded_outgoing_relations_for_frontier_at_version(
                    &frontier,
                    layout.kind,
                    plan.basis.version_id(),
                    work.remaining(),
                ),
            ApplicationQueryRootPathDirection::Reverse => runtime
                .read_truth()
                .bounded_incoming_relations_for_frontier_at_version(
                    &frontier,
                    layout.kind,
                    plan.basis.version_id(),
                    work.remaining(),
                ),
        }
        .map_err(|_| work_limit_denial(step.relation()))?;
        work.charge(
            read.adjacency_lists_read(),
            read.relation_records_examined(),
            read.endpoint_records_reserved(),
            step.relation(),
        )?;
        frontier = read
            .into_records()
            .into_iter()
            .map(|record| match step.direction() {
                ApplicationQueryRootPathDirection::Forward => record.target,
                ApplicationQueryRootPathDirection::Reverse => record.source,
            })
            .collect();
        apply_guards(
            runtime,
            graph,
            plan,
            path,
            step_index.saturating_add(1),
            &mut frontier,
            work,
        )?;
    }
    Ok(frontier)
}

fn apply_guards<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    path: &WorthQueryInstalledRootPath,
    after_step: usize,
    frontier: &mut BTreeSet<EntityId>,
    work: &mut RootSelectionWork,
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
        let read = read_governed_root_guard(
            runtime,
            computation,
            frontier,
            layout.entity_kind,
            &layout.locator,
            guard.expected(),
            plan.basis.version_id(),
            work.remaining(),
        )
        .map_err(|_| work_limit_denial(guard.field().as_str()))?;
        work.charge_predicate(
            read.entity_records_examined(),
            read.matching_entity_ids_reserved(),
            guard.field().as_str(),
        )?;
        *frontier = read.into_matching_entity_ids();
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn read_governed_root_guard(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    _projection: crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationInternalProjectionAdmission<'_>,
    frontier: &BTreeSet<EntityId>,
    entity_kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    expected: &worth_foundational::facade::AspectValue,
    version: worth_relational::facade::identity::VersionId,
    maximum_work: usize,
) -> Result<
    worth_relational::facade::runtime::BoundedFrontierFieldEqualityTruthRead,
    worth_relational::facade::runtime::FrontierFieldEqualityTruthReadLimitExceeded,
> {
    runtime
        .read_truth()
        .bounded_entity_field_equals_for_frontier_at_version(
            frontier,
            entity_kind,
            locator,
            expected,
            version,
            maximum_work,
        )
}

fn traversal_denial(subject: &str) -> WorthQueryApplicationReadExecutionDenial {
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
