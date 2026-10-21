//! Settled effectiveness depends on distinct selected identity, not edge count or adjacency clock.
use super::{
    coordinates, observation, stopped, unavailable, BTreeSet, Batch, Effect, EntityId, Lease,
    Locus, Outcome, PathDirection, QueryRequestExecution, Rule,
};
use crate::domain_computation::primary_graph::application_attempt::signed_creation::PublishedCertifiedSubject;
use crate::domain_computation::primary_graph::application_attempt::signed_selection::preserves_endpoint;

pub(in crate::domain_computation::primary_graph::application_attempt::signing_lifecycle) fn changed_selections(
    _rule: &Rule,
    subject: EntityId,
    footprint: &PublishedCertifiedSubject,
    lease: &Lease,
    effects: &[Effect],
    execution: &QueryRequestExecution<'_>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    batch: &mut Batch,
) -> Result<Vec<Locus>, Outcome> {
    if !effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::CreateRelation { .. }
                | Effect::DeleteRelation { .. }
                | Effect::DeleteEntity { .. }
        )
    }) {
        return Ok(Vec::new());
    }
    let captures: BTreeSet<_> = footprint
        .selections()
        .iter()
        .map(|selection| &selection.captured_field)
        .collect();
    let mut changed = Vec::new();
    for capture in captures {
        let mut anchor = subject;
        for selection in footprint
            .selections()
            .iter()
            .filter(|selection| &selection.captured_field == capture)
        {
            context.checkpoint(1).map_err(stopped)?;
            if selection.anchor != coordinates(anchor) {
                return Err(unavailable());
            }
            let kind = lease
                .layout
                .relation(&selection.relation)
                .ok_or_else(unavailable)?
                .kind;
            let edges = observation::edges(
                lease,
                anchor,
                kind,
                selection.direction,
                execution,
                request,
                context,
                batch,
            )?;
            let expected = edges
                .iter()
                .map(|edge| match selection.direction {
                    PathDirection::Forward => edge.to,
                    PathDirection::Reverse => edge.from,
                })
                .find(|target| coordinates(*target) == selection.selected)
                .ok_or_else(unavailable)?;
            if !preserves_endpoint(
                &edges,
                effects,
                anchor,
                kind,
                selection.direction,
                expected,
                context,
            )
            .map_err(stopped)?
            {
                let field = footprint
                    .source_for_capture(capture)
                    .ok_or_else(unavailable)?;
                if !changed.contains(field) {
                    changed.push(field.clone());
                }
                break;
            }
            anchor = expected;
        }
    }
    Ok(changed)
}
