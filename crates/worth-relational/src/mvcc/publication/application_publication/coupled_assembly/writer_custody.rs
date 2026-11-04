//! A participant's admission never transfers another writer's record custody.
use super::*;
use crate::{
    history::data::{source_scope_closure, CanonicalRequiredOutputOrigin},
    mvcc::RequiredOutputRolePosture as Posture,
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    transactions::data::{EntityMutationIntent, MutationIntent, RecordRef, RelationMutationIntent},
};

pub(super) fn require(
    execution: &ExecutionRequest<'_, '_>,
    custody: &RelationalAuthoredCoupledOwners,
    writer: Option<&CanonicalRequiredOutputOrigin>,
    plan: &crate::transactions::data::MergedCommitPlan,
) -> Result<std::collections::BTreeSet<crate::identity::data::RelationId>, TransactionCommitError> {
    let result = execution.run_native_with_checkpoint(|context| {
        let mut roots = Vec::new();
        if let Some(writer) = writer {
            for member in &writer.members {
                context.checkpoint(plan.merged_intents.len() as u64).map_err(stop)?;
                if member.posture != Posture::Retire && plan.merged_intents.iter().any(|intent| {
                    matches!(intent, MutationIntent::Entity(EntityMutationIntent::Delete(spec)) if spec.entity_id == member.id)
                }) {
                    roots.push(member.id);
                }
            }
        }
        let closure = if writer.is_some() {
            source_scope_closure::select(&roots, &custody.current, context)
                .map_err(|failure| match failure {
                    crate::history::data::CodecRetirementClosureStop::Request(cause) => stop(cause),
                    crate::history::data::CodecRetirementClosureStop::Payload(detail) => origin_denial(detail),
                })?
        } else {
            Vec::new()
        };
        let mut deferred_edges = std::collections::BTreeSet::new();
        for publication in &custody.current {
            let origin = &publication.origin;
            context.checkpoint((origin.relations.len() + closure.len() + custody.retirements.len() + 1) as u64).map_err(stop)?;
            let covered = writer.is_some_and(|writer| writer.meaning.same_route(&origin.meaning))
                || closure.iter().any(|p| Arc::ptr_eq(&p.origin, origin))
                || (writer.is_none() && custody.retirements.iter().any(|p| Arc::ptr_eq(&p.origin, origin)));
            if origin.source_scope_owned && !covered {
                deferred_edges.extend(origin.relations.iter().map(|edge| edge.id));
            }
        }
        for intent in &plan.merged_intents {
            context.checkpoint(1).map_err(stop)?;
            let target = match intent {
                MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => RecordRef::Entity(spec.entity_id),
                MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => RecordRef::Entity(spec.entity_id),
                MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => RecordRef::Relation(spec.relation_id),
                MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => RecordRef::Relation(spec.relation_id),
                _ => continue,
            };
            for publication in &custody.current {
                let origin = &publication.origin;
                context.checkpoint((origin.members.len() + origin.relations.len() + closure.len() + custody.retirements.len() + 1) as u64).map_err(stop)?;
                if !origin.source_scope_owned || origin.members.iter().all(|m| m.posture == Posture::Retire) {
                    continue;
                }
                let covered = writer.is_some_and(|writer| writer.meaning.same_route(&origin.meaning))
                    || closure.iter().any(|p| Arc::ptr_eq(&p.origin, origin))
                    || (writer.is_none() && custody.retirements.iter().any(|p| Arc::ptr_eq(&p.origin, origin)));
                let owned = match target {
                    RecordRef::Entity(id) => origin.members.iter().any(|m| m.id == id && m.posture != Posture::Retire),
                    RecordRef::Relation(id) => origin.relations.iter().any(|edge| edge.id == id),
                };
                if owned && !covered {
                    let mut error = PublicationError::authored_retirement(
                        Cause::OwnedPublicationRequiresCascade {
                            publication: origin.commit.commit_id, owner: origin.meaning.source_scope,
                        }, target.clone(),
                    );
                    if target != RecordRef::Entity(origin.meaning.source_scope) {
                        error.context = error.context.with_records(vec![target.clone(), RecordRef::Entity(origin.meaning.source_scope)]);
                    }
                    return Err(TransactionCommitError::publication(error));
                }
            }
        }
        Ok(deferred_edges)
    }).map_err(stop)?;
    result.0
}

fn stop(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    custody::execution_stop(cause)
}
