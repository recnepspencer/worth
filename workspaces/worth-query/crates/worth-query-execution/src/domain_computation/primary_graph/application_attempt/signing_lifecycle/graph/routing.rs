//! Follow only declared source paths affected by this candidate, with one deduplicated root bound.
use super::{
    add_subject, attempt, inverse_prefix, stopped, unavailable, BTreeMap, BTreeSet, Batch, Effect,
    EntityId, KindId, Lease, Locus, Outcome, PathDirection, QueryRequestExecution, Rule,
};
use worth_relational::facade::transactions::EntityReference;

pub(in crate::domain_computation::primary_graph::application_attempt::signing_lifecycle) fn affected_subjects(
    rule: &Rule,
    lease: &Lease,
    effects: &[Effect],
    changed: &BTreeMap<EntityId, Vec<Locus>>,
    execution: &QueryRequestExecution<'_>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    batch: &mut Batch,
) -> Result<BTreeSet<EntityId>, Outcome> {
    let mut subjects = BTreeSet::new();
    for (source, fields) in changed {
        for path in rule
            .paths()
            .iter()
            .filter(|path| fields.contains(&path.field))
        {
            for root in inverse_prefix(
                path.path.steps(),
                *source,
                lease,
                execution,
                request,
                context,
                batch,
            )? {
                add_subject(rule, &mut subjects, root)?;
            }
        }
    }
    for effect in effects {
        context.checkpoint(1).map_err(stopped)?;
        if let Effect::DeleteEntity { entity_id } = effect {
            let kind = entity_kind(lease, *entity_id, request)?;
            if Some(kind) == lease.layout.entity_kind(&rule.spec().source_entity) {
                add_subject(rule, &mut subjects, *entity_id)?;
            }
            for path in rule.paths() {
                for (index, hop) in path.path.steps().iter().enumerate() {
                    if Some(kind) == lease.layout.entity_kind(hop.child_entity()) {
                        for root in inverse_prefix(
                            &path.path.steps()[..=index],
                            *entity_id,
                            lease,
                            execution,
                            request,
                            context,
                            batch,
                        )? {
                            add_subject(rule, &mut subjects, root)?;
                        }
                    }
                }
            }
        }
        let relation = match effect {
            Effect::CreateRelation {kind, from, to, ..} => Some((*kind, from.existing(), to.existing())),
            Effect::DeleteRelation {relation_id} => Some(lease.handle().with_runtime(request, |runtime| {
                runtime.read_truth().project_snapshot(lease.snapshot()).ok_or_else(unavailable)?
                    .relation_record_with_projection_scope(request, *relation_id, worth_relational::facade::runtime::ProjectionAspectScope::empty(), |record| Some((record.kind_id(), Some(record.source()), Some(record.target()))))
                    .map_err(|cause| attempt(super::super::super::super::WorthQueryApplicationAttemptDenial::from(cause)))?.ok_or_else(unavailable)
            })?),
            _ => None,
        };
        let Some((kind, from, to)) = relation else {
            continue;
        };
        for path in rule.paths() {
            for (index, step) in path.path.steps().iter().enumerate() {
                if lease
                    .layout
                    .relation(step.relation())
                    .is_none_or(|relation| relation.kind != kind)
                {
                    continue;
                }
                let parent = match step.direction() {
                    PathDirection::Forward => from,
                    PathDirection::Reverse => to,
                };
                if let Some(parent) = parent {
                    for root in inverse_prefix(
                        &path.path.steps()[..index],
                        parent,
                        lease,
                        execution,
                        request,
                        context,
                        batch,
                    )? {
                        add_subject(rule, &mut subjects, root)?;
                    }
                }
            }
        }
    }
    Ok(subjects)
}
fn entity_kind(
    lease: &Lease,
    entity: EntityId,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<KindId, Outcome> {
    lease.handle().with_runtime(request, |runtime| {
        runtime
            .read_truth()
            .project_snapshot(lease.snapshot())
            .ok_or_else(unavailable)?
            .entity_record_with_projection_scope(
                request,
                entity,
                worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                |record| Some(record.kind_id()),
            )
            .map_err(|cause| {
                attempt(super::super::super::super::WorthQueryApplicationAttemptDenial::from(cause))
            })?
            .ok_or_else(unavailable)
    })
}
trait ExistingReference {
    fn existing(&self) -> Option<EntityId>;
}
impl ExistingReference for EntityReference {
    fn existing(&self) -> Option<EntityId> {
        match self {
            Self::Existing(entity) => Some(*entity),
            Self::Created(_) => None,
        }
    }
}
