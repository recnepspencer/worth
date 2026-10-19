//! Preserve exact selected identities and the native structural revision of each hop.
use super::{coordinates, identity, BodyCandidate, CaptureSpec, Denial, Selection};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection as Direction, WorthQueryApplicationObservedFact as Fact,
};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_query::ApplicationQueryRootPathDirection as PathDirection,
    application_schema::ApplicationSchema,
};
use worth_relational::facade::{
    identity::EntityId, runtime::RelationalAdjacencyDirection as NativeDirection,
};

pub(super) fn capture<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
    candidate: &BodyCandidate<Schema, Binding>,
    mut current: EntityId,
    spec: &CaptureSpec<'_>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    selections: &mut Vec<Selection>,
) -> Result<EntityId, Denial> {
    let Some(path) = spec.path else {
        return Ok(current);
    };
    if !path.guards().is_empty() {
        return Err(Denial::CandidateMismatch);
    }
    for hop in path.steps() {
        let relation = candidate
            .read_set
            .lease
            .layout
            .relation(hop.relation())
            .ok_or(Denial::CandidateMismatch)?;
        let (direction, native_direction) = match hop.direction() {
            PathDirection::Forward => (Direction::Outgoing, NativeDirection::Outgoing),
            PathDirection::Reverse => (Direction::Incoming, NativeDirection::Incoming),
        };
        let mut selected = None;
        let mut observed = None;
        for fact in candidate.read_set.facts.iter() {
            identity::admit_assembly(context, 0)?;
            if let Fact::Adjacency {
                relation_kind,
                anchor,
                direction: observed_direction,
                relations,
                ..
            } = fact
                && *relation_kind == relation.kind
                && *anchor == current
                && *observed_direction == direction
            {
                observed = Some(relations);
                for edge in relations {
                    identity::admit_assembly(context, 0)?;
                    let endpoint = match direction {
                        Direction::Outgoing => edge.to,
                        Direction::Incoming => edge.from,
                    };
                    if selected.is_some_and(|previous| previous != endpoint) {
                        return Err(Denial::CandidateMismatch);
                    }
                    selected = Some(endpoint);
                }
            }
        }
        let next = selected.ok_or(Denial::CandidateMismatch)?;
        let preserves = crate::domain_computation::primary_graph::application_attempt::signed_selection::preserves_endpoint(
            observed.ok_or(Denial::CandidateMismatch)?, &candidate.effects, current, relation.kind, hop.direction(), next, context,
        ).map_err(|cause| Denial::Execution(crate::domain_computation::primary_graph::application_contribution::advancement_denial(worth_execution::WorkCeilingDenial::Stopped(cause))))?;
        if !preserves {
            return Err(Denial::CandidateMismatch);
        }
        let revision = candidate.read_set.lease.handle().with_runtime(request, |runtime| {
            runtime.read_truth().project_snapshot(candidate.read_set.lease.snapshot())
                .ok_or(Denial::CandidateMismatch)?
                .bounded_adjacency_structural_revision(request, current, relation.kind, native_direction, 1)
                .map_err(|cause| match cause {
                    worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::Execution(cause) => Denial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::from(cause)),
                    _ => Denial::CandidateMismatch,
                })?
                .revision().ok_or(Denial::CandidateMismatch)
        })?;
        identity::admit_assembly(
            context,
            Selection::quote(spec.target, hop.relation()).ok_or(Denial::RepresentationCapacity)?,
        )?;
        selections.push(Selection {
            captured_field: spec.target.clone(),
            anchor: coordinates(current),
            selected: coordinates(next),
            relation: hop.relation().into(),
            direction: hop.direction(),
            native_revision: revision.0,
        });
        current = next;
    }
    Ok(current)
}
