use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::indexes::DerivedIndexGenerationId;

use super::{
    read_execution_denial, OneShotReadWorkObservation, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};
use crate::domain_computation::primary_graph::application_query::read_execution::read_plan::ReadPlan;

mod evidence;
mod path_union;

use evidence::RootPathSourceBuilder;

pub(super) struct BoundedRootSelection {
    pub(super) candidates: Vec<EntityId>,
    pub(super) selected_predicate_source:
        Option<super::super::observed_source::WorthQueryObservedFieldRevision>,
    pub(super) root_path_source: Option<
        std::collections::BTreeMap<
            EntityId,
            std::sync::Arc<super::super::observed_source::WorthQueryObservedRootSelection>,
        >,
    >,
    pub(super) result_set_source:
        Option<std::sync::Arc<super::super::observed_source::WorthQueryObservedRootSelection>>,
    pub(super) examined_candidates: usize,
    pub(super) predicate_work_units: usize,
    pub(super) work_units: usize,
    pub(super) predicate_index_generation: Option<DerivedIndexGenerationId>,
    pub(super) adjacency_lists_read: usize,
    pub(super) relation_records_examined: usize,
}

mod work;
use work::RootSelectionWork;

pub(super) fn select_bounded_roots(
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,

    capture_result_set: bool,
    spent: Option<&OneShotReadWorkObservation<'_>>,

    interruption: super::ReadInterruption<'_>,
) -> Result<BoundedRootSelection, WorthQueryApplicationReadExecutionDenial> {
    interruption.checkpoint(plan.name)?;
    let contract = plan.contract;
    if !contract.root_paths().is_empty() {
        return path_union::select_root_path_union(
            execution,
            runtime,
            graph,
            plan,
            contract.root_paths(),
            capture_result_set,
            spent,
            interruption,
        );
    }
    match contract.predicates() {
        [] => {
            if let Some(spent) = spent {
                spent.charge(1, plan.name)?;
                spent.root(1);
            }
            let result_set_source = if capture_result_set {
                let mut source = RootPathSourceBuilder::default();
                source.observe_entity(plan.root);
                Some(source.finish()?)
            } else {
                None
            };
            Ok(BoundedRootSelection {
                candidates: vec![plan.root],
                selected_predicate_source: None,
                root_path_source: None,
                result_set_source,
                examined_candidates: 1,
                predicate_work_units: 1,
                work_units: 1,
                predicate_index_generation: None,
                adjacency_lists_read: 0,
                relation_records_examined: 0,
            })
        }
        [predicate] => select_scoped_root(
            runtime,
            execution,
            graph,
            plan,
            predicate,
            capture_result_set,
            spent,
            interruption,
        ),
        _ => Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
            plan.name,
        )),
    }
}

fn select_scoped_root(
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    predicate: &worth_query_installation::facade::WorthQueryInstalledGraphPredicate,

    capture_result_set: bool,
    spent: Option<&OneShotReadWorkObservation<'_>>,

    interruption: super::ReadInterruption<'_>,
) -> Result<BoundedRootSelection, WorthQueryApplicationReadExecutionDenial> {
    let (entity, aspect, field) = predicate.field();
    let computation = plan
        .governance
        .admit_internal_projection(
            predicate.field(),
            predicate.field_key(),
            ApplicationQueryObservableInfluence::RowPresence,
        )
        .ok_or_else(|| {
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
                field,
            )
        })?;
    let layout = graph.equality_field(entity, aspect, field).ok_or_else(|| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
            field,
        )
    })?;
    if !computation.admits_locator(&layout.locator) {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
            field,
        ));
    }
    let expected = plan
        .parameters
        .bindings()
        .iter()
        .find(|(name, _)| *name == predicate.parameter())
        .map(|(_, value)| value)
        .ok_or_else(|| {
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
                predicate.parameter(),
            )
        })?;
    let mut work = RootSelectionWork::new(interruption, spent);
    let projection = runtime.project_snapshot(plan.snapshot).ok_or_else(|| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
            field,
        )
    })?;
    let mut set_source = RootPathSourceBuilder::default();
    // Observe both matching and filtered subjects: a later guard change can
    // change row presence even when the current result is empty.
    let observed = {
        set_source.observe_entity(plan.root);
        set_source.observe_guard_field(
            execution,
            &projection,
            graph,
            plan.root,
            entity,
            predicate.aspect_key(),
            predicate.field_key(),
            &mut work,
        )?
    };
    interruption.checkpoint(field)?;
    let frontier = std::collections::BTreeSet::from([plan.root]);
    let lookup = path_union::read_governed_root_guard(
        execution,
        runtime,
        computation,
        &frontier,
        layout.entity_kind,
        &layout.locator,
        expected,
    )
    .map_err(|stop| path_union::frontier_denial(stop, field))?;
    work.charge_predicate(
        lookup.entity_records_examined(),
        lookup.matching_entity_ids_reserved(),
        field,
    )?;
    let examined_candidates = lookup.entity_records_examined();
    let scoped = lookup.into_matching_entity_ids().into_iter().next();
    let selected_predicate_source = scoped.map(|_| observed);
    let result_set_source = capture_result_set
        .then(|| set_source.finish())
        .transpose()?;
    Ok(BoundedRootSelection {
        candidates: scoped.into_iter().collect(),
        selected_predicate_source,
        root_path_source: None,
        result_set_source,
        examined_candidates,
        predicate_work_units: work.predicate_work_units,
        work_units: work.work_units,
        predicate_index_generation: None,
        adjacency_lists_read: 0,
        relation_records_examined: 0,
    })
}
