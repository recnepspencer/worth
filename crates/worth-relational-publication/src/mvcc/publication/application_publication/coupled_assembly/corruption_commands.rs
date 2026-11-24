//! Modify existing writer commands for the closed participant-corruption receiver.
use super::super::{RelationalRequiredOutputPublication, RequiredOutputPublication};
use super::producer_posture::RelationalCoupledProducerPosture;
use super::refusals::reject;
use super::retained_assembly::CoupledPublicationPart;
use contracts::identity::data::record_ids::EntityId;
use contracts::schema::data::RelationalSchemaRegistry;
use contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::WorkerIntentBatch;
use worth_relational_contracts::internal_port as contracts;

pub fn delete_entity(
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
                    contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::CorruptPublication,
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
            contracts::transactions::data::primitives::EntityReference::Existing(id) => *id,
            contracts::transactions::data::primitives::EntityReference::Created(reference) => {
                created.resolve(reference).ok_or_else(|| {
                    reject(
                        contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::CorruptPublication,
                        authored_source,
                    )
                })?
            }
        }
    };
    let batch = batches.first_mut().expect("actual writer batch");
    batch.intents.push(
        contracts::transactions::data::intents::mutation_intent::MutationIntent::Entity(
            contracts::transactions::data::intents::mutation_intent::EntityMutationIntent::Delete(
                contracts::transactions::data::intents::mutation_intent::DeleteEntityIntent {
                    entity_id: target,
                },
            ),
        ),
    );
    Ok(())
}

pub fn delete_membership(
    predecessor: &RelationalRequiredOutputPublication,
    registry: &RelationalSchemaRegistry,
    batches: &mut [WorkerIntentBatch],
) -> Result<(), TransactionCommitError> {
    let edge = predecessor
        .origin
        .relations
        .iter()
        .find(|edge| {
            (*registry.owner_relation_kinds())
                .get(&edge.kind)
                .is_some_and(|kind| kind.kind_name.starts_with("worth-aggregate-membership-"))
        })
        .ok_or_else(|| {
            reject(
                contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::CorruptPublication,
                predecessor.origin.meaning.source_scope,
            )
        })?;
    let Some(batch) = batches.first_mut() else {
        return Err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "real coupled writer has no command batch",
            ),
        );
    };
    batch.intents.push(
        contracts::transactions::data::intents::mutation_intent::MutationIntent::Relation(
            contracts::transactions::data::intents::mutation_intent::RelationMutationIntent::Delete(
                contracts::transactions::data::intents::mutation_intent::DeleteRelationIntent {
                    relation_id: edge.id,
                },
            ),
        ),
    );
    Ok(())
}

mod recorded_predecessor;
pub use recorded_predecessor::corrupt_predecessor_for_test;
