//! One real all-create output group with a retained endpoint.
use crate::identity::data::{KindId, PartitionId};
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, EntityReference, EntitySpec, MutationIntent, RelationSpec,
};

pub(crate) fn create_group_with_external_endpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    external: crate::identity::data::EntityId,
) -> crate::transactions::data::CommitResult {
    let target = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("internal-endpoint"),
    };
    let batch = crate::transactions::data::WorkerIntentBatch::new("open-generated-group")
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: crate::tests::support::name_field_patch("internal-endpoint"),
        })))
        .push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("external-edge"),
                source: EntityReference::Existing(external),
                target: EntityReference::Created(target),
                fields: crate::tests::support::relation_label_field_patch("external-edge"),
            },
        )));
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the open generated group fits transaction budgets");
    transaction
        .commit(execution, runtime)
        .expect("the open generated group is otherwise a valid all-create publication")
}
