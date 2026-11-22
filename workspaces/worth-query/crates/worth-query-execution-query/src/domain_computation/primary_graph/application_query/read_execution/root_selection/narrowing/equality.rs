//! Root-entity `where_equal` predicates narrow a root-path union through the
//! governed equality reader, observing every examined candidate's field.
use super::{NarrowingRead, RootPathSourceBuilder, RootSelectionWork};
use crate::domain_computation::primary_graph::application_query::read_execution::{
    read_execution_denial, root_selection::path_union, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};
use std::collections::BTreeSet;
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_relational::facade::identity::EntityId;

pub fn narrow<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    candidates: Vec<EntityId>,
    evidence: &mut RootPathSourceBuilder,
    work: &mut RootSelectionWork<'_>,
) -> Result<Vec<EntityId>, WorthQueryApplicationReadExecutionDenial> {
    let plan = read.plan;
    let mut frontier: BTreeSet<EntityId> = candidates.into_iter().collect();
    for predicate in plan.contract.predicates() {
        let (entity, aspect, field) = predicate.field();
        if entity != plan.contract.root_entity() {
            return Err(plan.predicate_unavailable_denial());
        }
        let admitted = plan
            .governance
            .admit_internal_projection(
                predicate.field(),
                predicate.field_key(),
                ApplicationQueryObservableInfluence::RowPresence,
            )
            .ok_or_else(|| projection_unavailable(field))?;
        let layout = read
            .graph
            .equality_field(entity, aspect, field)
            .ok_or_else(|| plan.predicate_unavailable_denial())?;
        if !admitted.admits_locator(&layout.locator) {
            return Err(projection_unavailable(field));
        }
        let expected = plan
            .parameters
            .bindings()
            .iter()
            .find(|(name, _)| name.as_ref() == predicate.parameter())
            .map(|(_, value)| value)
            .ok_or_else(|| plan.predicate_unavailable_denial())?;
        for root in &frontier {
            evidence.observe_guard_field(
                read.execution,
                read.projection,
                read.graph,
                *root,
                entity,
                predicate.aspect_key(),
                predicate.field_key(),
                work,
            )?;
        }
        work.checkpoint(field)?;
        let lookup = path_union::read_governed_root_guard(
            read.execution,
            read.runtime,
            admitted,
            &frontier,
            layout.entity_kind,
            &layout.locator,
            expected,
        )
        .map_err(|stop| path_union::frontier_denial(stop, field))?;
        work.charge_predicate(
            lookup.entity_records_examined(),
            lookup.matching_entity_ids_reserved(),
            lookup.field_reads(),
            field,
        )?;
        frontier = lookup.into_matching_entity_ids().into_iter().collect();
    }
    Ok(frontier.into_iter().collect())
}

/// Governance refusal is projection unavailability, as for the exact-root guard.
fn projection_unavailable(field: &str) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
        field,
    )
}
