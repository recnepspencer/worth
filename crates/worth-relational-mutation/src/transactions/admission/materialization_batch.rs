use crate::contracts::transactions::data::intents::materialization_intent::{
    MaterializationMutationIntent, RematerializeEntityIntent, RematerializeRelationIntent,
    SuspendEntityMaterializationIntent, SuspendRelationMaterializationIntent,
};
use crate::contracts::transactions::data::intents::mutation_intent::MutationIntent;
use crate::contracts::transactions::data::primitives::WorkerIntentBatch;
use worth_relational_branch::internal_port::branch::materialization::{
    RelationalEntityMaterialization, RelationalMaterializationRecord,
    RelationalRelationMaterialization,
};

pub fn suspension_batch(records: &[RelationalMaterializationRecord]) -> WorkerIntentBatch {
    records.iter().fold(
        WorkerIntentBatch::new("owner-materialization-suspension"),
        |batch, record| {
            batch.push(MutationIntent::Materialization(match record {
                RelationalMaterializationRecord::Entity { entity_id, .. } => {
                    MaterializationMutationIntent::SuspendEntity(
                        SuspendEntityMaterializationIntent {
                            entity_id: *entity_id,
                        },
                    )
                }
                RelationalMaterializationRecord::Relation { relation_id, .. } => {
                    MaterializationMutationIntent::SuspendRelation(
                        SuspendRelationMaterializationIntent {
                            relation_id: *relation_id,
                        },
                    )
                }
            }))
        },
    )
}

pub fn rematerialization_batch(
    entities: Vec<RelationalEntityMaterialization>,
    relations: Vec<RelationalRelationMaterialization>,
) -> WorkerIntentBatch {
    let batch = entities.into_iter().fold(
        WorkerIntentBatch::new("owner-materialization-restoration"),
        |batch, spec| {
            batch.push(MutationIntent::Materialization(
                MaterializationMutationIntent::RematerializeEntity(RematerializeEntityIntent {
                    entity_id: spec.entity_id,
                    kind_id: spec.kind_id,
                    fields: spec.fields,
                }),
            ))
        },
    );
    relations.into_iter().fold(batch, |batch, spec| {
        batch.push(MutationIntent::Materialization(
            MaterializationMutationIntent::RematerializeRelation(RematerializeRelationIntent {
                relation_id: spec.relation_id,
                kind_id: spec.kind_id,
                source: spec.source,
                target: spec.target,
                fields: spec.fields,
            }),
        ))
    })
}
