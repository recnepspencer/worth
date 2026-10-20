//! Source identity and selection changes route through the inverse of the authored capture paths.
mod observation;
mod routing;
mod selection;
use super::consequences::Batch;
use super::{
    attempt, capacity, coordinates, stopped, unavailable, Effect, EntityId, Fact, Lease, Locus,
    Outcome, QueryRequestExecution, Rule,
};
pub(super) use routing::affected_subjects;
pub(super) use selection::changed_selections;
use std::collections::{BTreeMap, BTreeSet};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryRootPathDirection as PathDirection, ApplicationQueryRootPathStep as Step,
};
use worth_relational::facade::identity::KindId;

fn add_subject(
    rule: &Rule,
    subjects: &mut BTreeSet<EntityId>,
    subject: EntityId,
) -> Result<(), Outcome> {
    if !subjects.contains(&subject)
        && let Some(maximum) = rule.maximum_subjects()
        && subjects.len() >= maximum
    {
        return Err(super::refused(
            super::Kind::SigningLifecycleSubjectCapacity {
                maximum_subjects: maximum,
            },
        ));
    }
    subjects.insert(subject);
    Ok(())
}

fn inverse_prefix(
    steps: &[Step],
    endpoint: EntityId,
    lease: &Lease,
    execution: &QueryRequestExecution<'_>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    batch: &mut Batch,
) -> Result<BTreeSet<EntityId>, Outcome> {
    let mut frontier = BTreeSet::from([endpoint]);
    for step in steps.iter().rev() {
        let relation = lease
            .layout
            .relation(step.relation())
            .ok_or_else(unavailable)?;
        let direction = match step.direction() {
            PathDirection::Forward => PathDirection::Reverse,
            PathDirection::Reverse => PathDirection::Forward,
        };
        let mut previous = BTreeSet::new();
        for anchor in frontier {
            for edge in observation::edges(
                lease,
                anchor,
                relation.kind,
                direction,
                execution,
                request,
                context,
                batch,
            )? {
                context.checkpoint(1).map_err(stopped)?;
                previous.insert(match direction {
                    PathDirection::Forward => edge.to,
                    PathDirection::Reverse => edge.from,
                });
            }
        }
        frontier = previous;
    }
    Ok(frontier)
}
