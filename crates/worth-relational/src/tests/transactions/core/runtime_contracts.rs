use crate::facade::durability::{DurabilityError, RecoveryFailureClass};
use crate::facade::errors::{RelationalError, RelationalSubsystem};
use crate::facade::history::BranchCreateError;
use crate::facade::identity::{EntityId, EntityStorageId, RelationId, RelationStorageId};
use crate::facade::publication::{PublicationError, PublicationStage};
use crate::facade::replay::{ReplayError, ReplayFailureClass};
use crate::facade::schema::SchemaRegistryError;
use crate::facade::transactions::{EntitySpec, MutationIntent};
use crate::tests::support::*;

#[test]
fn harness_defaults_require_determinism_and_parity() {
    let expectations = crate::facade::harness::default_harness_expectations();
    assert!(expectations.serial_parallel_parity_required);
}

#[test]
fn tagged_record_ids_preserve_storage_identity() {
    let entity_id = EntityId::new(PartitionId(7), 11, 3);
    let relation_id = RelationId::new(PartitionId(9), 13, 4);

    let entity_storage: EntityStorageId = entity_id.storage_id();
    let relation_storage: RelationStorageId = relation_id.storage_id();

    assert_eq!(entity_storage.partition_id, PartitionId(7));
    assert_eq!(entity_storage.local_slot.0, 11);
    assert_eq!(relation_storage.partition_id, PartitionId(9));
    assert_eq!(relation_storage.local_slot.0, 13);
    assert_ne!(entity_id.partition_id, relation_id.partition_id);
}

#[test]
fn relational_error_wraps_authority_failures_with_context() {
    let runtime = runtime_with_test_schema();
    let entity = create_entity(&runtime, "first");
    delete_entity(&runtime, entity);

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("stale-update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "stale",
                ),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let transaction_error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    let wrapped: RelationalError = transaction_error.into();
    assert!(matches!(wrapped, RelationalError::Transaction(_)));
    assert_eq!(
        wrapped.context().subsystem,
        RelationalSubsystem::Transaction
    );

    let wrapped: RelationalError = SchemaRegistryError::unknown_entity_kind(KindId(999)).into();
    assert!(matches!(wrapped, RelationalError::Schema(_)));
    assert_eq!(wrapped.context().subsystem, RelationalSubsystem::Schema);

    let wrapped: RelationalError = BranchCreateError::branch_already_exists().into();
    assert!(matches!(wrapped, RelationalError::History(_)));
    assert_eq!(wrapped.context().subsystem, RelationalSubsystem::History);

    let wrapped: RelationalError =
        PublicationError::new(PublicationStage::Visibility, "publication failed").into();
    assert!(matches!(wrapped, RelationalError::Publication(_)));

    let wrapped: RelationalError =
        DurabilityError::new(RecoveryFailureClass::DurableIoFailure, "durability failed").into();
    assert!(matches!(wrapped, RelationalError::Durability(_)));

    let wrapped: RelationalError =
        ReplayError::new(ReplayFailureClass::SchemaMismatch, "replay failed").into();
    assert!(matches!(wrapped, RelationalError::Replay(_)));
}

#[test]
fn transaction_intent_is_the_shared_mutation_intent_type() {
    let create = MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("alias"),
        fields: crate::tests::support::single_string_aspect_field_patch(
            crate::tests::support::aspect_key("name"),
            crate::tests::support::field_key("name"),
            "alias",
        ),
    }));
    let transaction_intent: MutationIntent = create.clone();

    assert_eq!(transaction_intent, create);
}
