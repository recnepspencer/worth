//! Modify existing writer commands for the closed participant-corruption receiver.
use super::super::{RelationalRequiredOutputPublication, RequiredOutputPublication};
use super::producer_posture::RelationalCoupledProducerPosture;
use super::refusals::reject;
use super::retained_assembly::CoupledPublicationPart;
use crate::identity::data::EntityId;
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::outcomes::{
    created_entity_bindings::CommitCreatedEntityBindings,
    transaction_commit_error::TransactionCommitError,
};
use crate::transactions::data::primitives::WorkerIntentBatch;

pub(super) fn delete_entity(
    posture: &RelationalCoupledProducerPosture,
    parts: &[CoupledPublicationPart],
    created: &CommitCreatedEntityBindings,
    authored_source: EntityId,
    batches: &mut [WorkerIntentBatch],
    delete_source: bool,
) -> Result<(), TransactionCommitError> {
    let target = if delete_source {
        posture
            .producing_meaning()
            .expect("actual writer route")
            .source_scope
    } else {
        let prior = parts
            .first()
            .and_then(|part| part.publication.owned())
            .ok_or_else(|| {
                reject(
                    crate::publication::data::AuthoredRetirementRefusal::CorruptPublication,
                    authored_source,
                )
            })?;
        let roles = match prior {
            RequiredOutputPublication::Birth { roles, .. }
            | RequiredOutputPublication::Preserve { roles, .. } => roles,
            _ => unreachable!("a real producer part"),
        };
        let role = roles
            .first()
            .expect("the actual producer has its output role");
        match &role.target {
            crate::transactions::data::primitives::EntityReference::Existing(id) => *id,
            crate::transactions::data::primitives::EntityReference::Created(reference) => {
                created.resolve(reference).ok_or_else(|| {
                    reject(
                        crate::publication::data::AuthoredRetirementRefusal::CorruptPublication,
                        authored_source,
                    )
                })?
            }
        }
    };
    let batch = batches.first_mut().expect("actual writer batch");
    batch.intents.push(
        crate::transactions::data::intents::mutation_intent::MutationIntent::Entity(
            crate::transactions::data::intents::mutation_intent::EntityMutationIntent::Delete(
                crate::transactions::data::intents::mutation_intent::DeleteEntityIntent {
                    entity_id: target,
                },
            ),
        ),
    );
    Ok(())
}

pub(super) fn delete_membership(
    predecessor: &RelationalRequiredOutputPublication,
    registry: &RelationalSchemaRegistry,
    batches: &mut [WorkerIntentBatch],
) -> Result<(), TransactionCommitError> {
    let edge = predecessor
        .origin
        .relations
        .iter()
        .find(|edge| {
            registry
                .relation_kinds
                .get(&edge.kind)
                .is_some_and(|kind| kind.kind_name.starts_with("worth-aggregate-membership-"))
        })
        .ok_or_else(|| {
            reject(
                crate::publication::data::AuthoredRetirementRefusal::CorruptPublication,
                predecessor.origin.meaning.source_scope,
            )
        })?;
    let Some(batch) = batches.first_mut() else {
        return Err(
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "real coupled writer has no command batch",
            ),
        );
    };
    batch.intents.push(
        crate::transactions::data::intents::mutation_intent::MutationIntent::Relation(
            crate::transactions::data::intents::mutation_intent::RelationMutationIntent::Delete(
                crate::transactions::data::intents::mutation_intent::DeleteRelationIntent {
                    relation_id: edge.id,
                },
            ),
        ),
    );
    Ok(())
}
