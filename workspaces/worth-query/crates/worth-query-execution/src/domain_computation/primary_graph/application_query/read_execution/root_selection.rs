use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupRequest, BoundedIndexParityMode, DerivedIndexGenerationId,
};

use super::{
    read_execution_denial, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;

mod evidence;
mod path_union;

pub(super) struct BoundedRootSelection {
    pub(super) candidates: Vec<EntityId>,
    pub(super) selected_predicate_source:
        Option<super::super::observed_source::WorthQueryObservedAspectRevision>,
    pub(super) root_path_source: Option<
        std::collections::BTreeMap<
            EntityId,
            std::sync::Arc<super::super::observed_source::WorthQueryObservedRootSelection>,
        >,
    >,
    pub(super) examined_candidates: usize,
    pub(super) predicate_work_units: usize,
    pub(super) work_units: usize,
    pub(super) predicate_index_generation: Option<DerivedIndexGenerationId>,
    pub(super) adjacency_lists_read: usize,
    pub(super) relation_records_examined: usize,
}

pub(super) struct RootSelectionWork {
    maximum_work: usize,
    work_units: usize,
    adjacency_lists_read: usize,
    relation_records_examined: usize,
    predicate_records_examined: usize,
    predicate_work_units: usize,
}

impl RootSelectionWork {
    fn new(maximum_work: usize) -> Self {
        Self {
            maximum_work,
            work_units: 0,
            adjacency_lists_read: 0,
            relation_records_examined: 0,
            predicate_records_examined: 0,
            predicate_work_units: 0,
        }
    }

    fn remaining(&self) -> usize {
        self.maximum_work.saturating_sub(self.work_units)
    }

    fn charge(
        &mut self,
        adjacency_lists_read: usize,
        relation_records_examined: usize,
        endpoint_records_reserved: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        let charged = adjacency_lists_read
            .saturating_add(relation_records_examined)
            .saturating_add(endpoint_records_reserved);
        if self.work_units.saturating_add(charged) > self.maximum_work {
            return Err(read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                subject,
            ));
        }
        self.work_units = self.work_units.saturating_add(charged);
        self.adjacency_lists_read = self
            .adjacency_lists_read
            .saturating_add(adjacency_lists_read);
        self.relation_records_examined = self
            .relation_records_examined
            .saturating_add(relation_records_examined);
        Ok(())
    }

    fn charge_predicate(
        &mut self,
        records_examined: usize,
        matches_reserved: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        let charged = records_examined.saturating_add(matches_reserved);
        if self.work_units.saturating_add(charged) > self.maximum_work {
            return Err(read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                subject,
            ));
        }
        self.work_units = self.work_units.saturating_add(charged);
        self.predicate_records_examined = self
            .predicate_records_examined
            .saturating_add(records_examined);
        self.predicate_work_units = self.predicate_work_units.saturating_add(charged);
        Ok(())
    }

    fn charge_source_observation(
        &mut self,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if self.work_units >= self.maximum_work {
            return Err(read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                subject,
            ));
        }
        self.work_units += 1;
        Ok(())
    }

    fn charge_source_copy(
        &mut self,
        units: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if self.work_units.saturating_add(units) > self.maximum_work {
            return Err(read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                subject,
            ));
        }
        self.work_units += units;
        Ok(())
    }
}

pub(super) fn select_bounded_roots<
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
    result_buffer: &mut super::super::resource_lifecycle::WorthQueryApplicationResultBufferReservation,
) -> Result<BoundedRootSelection, WorthQueryApplicationReadExecutionDenial> {
    let contract = plan.query.read_family_binding().planning_contract();
    if !contract.root_paths().is_empty() {
        return path_union::select_root_path_union(
            runtime,
            graph,
            plan,
            contract.root_paths(),
            result_buffer,
        );
    }
    match contract.predicates() {
        [] => Ok(BoundedRootSelection {
            candidates: vec![plan.scope.entity_id()],
            selected_predicate_source: None,
            root_path_source: None,
            examined_candidates: 1,
            predicate_work_units: 1,
            work_units: 1,
            predicate_index_generation: None,
            adjacency_lists_read: 0,
            relation_records_examined: 0,
        }),
        [predicate] => select_indexed_root(runtime, graph, plan, predicate),
        _ => Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
            plan.query.name(),
        )),
    }
}

fn select_indexed_root<
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
    predicate: &worth_query_installation::facade::WorthQueryInstalledGraphPredicate,
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
        .map(|(_, value)| value.clone())
        .ok_or_else(|| {
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
                predicate.parameter(),
            )
        })?;
    let equality_index_id = layout.equality_index_id.ok_or_else(|| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
            field,
        )
    })?;
    let request = BoundedEntityFieldLookupRequest::new(
        plan.basis.snapshot_handle().clone(),
        equality_index_id,
        layout.entity_kind,
        layout.locator.clone(),
        expected,
        2,
    )
    .map_err(|_| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
            field,
        )
    })?;
    let lookup =
        execute_governed_predicate_lookup(runtime, computation, request).map_err(|_| {
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
                field,
            )
        })?;
    let scoped = lookup
        .candidate_entity_ids()
        .iter()
        .copied()
        .find(|candidate| *candidate == plan.scope.entity_id());
    if scoped.is_none() && lookup.overflowed() {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::PredicateLookupOverflow,
            field,
        ));
    }
    let selected_predicate_source = scoped
        .map(|entity_id| {
            let contract_revision = graph
                .aspect_contract(entity, predicate.aspect_key())
                .ok_or_else(|| {
                    read_execution_denial(
                        WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
                        field,
                    )
                })?
                .revision();
            let native_revision = runtime
                .read_truth()
                .project_snapshot(plan.basis.snapshot_handle())
                .and_then(|view| view.entity_aspect_version(entity_id, predicate.aspect_key()))
                .ok_or_else(|| {
                    read_execution_denial(
                        WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
                        field,
                    )
                })?;
            Ok(
                super::super::observed_source::WorthQueryObservedAspectRevision {
                    entity: entity_id,
                    entity_name: entity.to_owned(),
                    aspect: predicate.aspect_key().clone(),
                    contract_revision,
                    native_revision,
                },
            )
        })
        .transpose()?;
    Ok(BoundedRootSelection {
        candidates: scoped.into_iter().collect(),
        selected_predicate_source,
        root_path_source: None,
        examined_candidates: lookup.examined_entry_count(),
        predicate_work_units: lookup.examined_entry_count(),
        work_units: lookup.examined_entry_count(),
        predicate_index_generation: Some(lookup.generation_id()),
        adjacency_lists_read: 0,
        relation_records_examined: 0,
    })
}

fn execute_governed_predicate_lookup(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    _projection: super::super::disclosure::WorthQueryApplicationInternalProjectionAdmission<'_>,
    request: BoundedEntityFieldLookupRequest,
) -> Result<
    worth_relational::facade::indexes::BoundedEntityFieldLookupOutcome,
    worth_relational::facade::indexes::BoundedEntityFieldLookupDenial,
> {
    runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production)
}
