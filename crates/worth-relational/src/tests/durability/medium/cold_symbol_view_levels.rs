//! Requested symbol closures stay local even when another fork warmed Full.
use super::*;
use crate::transactions::data::{EntityMutationIntent, UpdateEntityFieldsIntent};
use worth_execution::ExecutionImmutableBytes;

pub(in super::super) fn admit_population(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    producer: &crate::runtime::RelationalRuntime,
) {
    let (_, basis) = producer
        .observe_branch(&producer.main_branch_identity())
        .unwrap();
    for index in 0..=600 {
        let text = format!("paged-view-value-{index:04}");
        let input = ExecutionImmutableBytes::from_external_bytes(Arc::new(text.as_bytes().into()));
        producer
            .admit_field_text(execution, &basis, &input)
            .unwrap();
    }
    drop(basis);
}

pub(in super::super) enum TextMutation {
    Create,
    Update(crate::identity::data::EntityId),
}

pub(in super::super) fn publish_text(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    mutation: TextMutation,
    text: &str,
) -> (
    crate::identity::data::EntityId,
    crate::snapshots::data::SnapshotHandle,
) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let input = ExecutionImmutableBytes::from_external_bytes(Arc::new(text.as_bytes().into()));
    let text = runtime.admit_field_text(execution, &basis, &input).unwrap();
    let fields = AspectFieldPatch::from_locator(
        crate::transactions::data::planned_single_field_locator(
            aspect_key("name"),
            field_key("name"),
        ),
        text.field_value_for(&basis).unwrap(),
    );
    let batch = match mutation {
        TextMutation::Create => {
            let mut batch = batch_create("symbol-view-record");
            let MutationIntent::Create(CreateIntent::Entity(entity)) = &mut batch.intents[0] else {
                panic!("producer declares one entity create");
            };
            entity.fields = fields;
            batch
        }
        TextMutation::Update(entity_id) => {
            WorkerIntentBatch::new("symbol-view-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
            ))
        }
    };
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let outcome = transaction.commit(execution, runtime).unwrap();
    let entity = changed_entities(&outcome)[0];
    (entity, outcome.snapshot.clone())
}
