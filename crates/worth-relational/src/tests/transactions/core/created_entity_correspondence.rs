use crate::facade::symbols::ClientKey;
use crate::facade::transactions::{CreatedEntityRef, EntitySpec};
use crate::tests::support::*;

#[test]
fn committed_create_references_resolve_their_own_distinct_persisted_meanings() {
    let runtime = runtime_with_test_schema();
    let partition_id = PartitionId::main();
    let kind_id = KindId(1);
    let first = CreatedEntityRef {
        partition_id,
        kind_id,
        client_key: ClientKey::raw("first-owner-reference"),
    };
    let second = CreatedEntityRef {
        partition_id,
        kind_id,
        client_key: ClientKey::raw("second-owner-reference"),
    };
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("created-reference-correspondence")
                .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id,
                    kind_id,
                    client_key: first.client_key.clone(),
                    fields: name_field_patch("first-persisted-meaning"),
                })))
                .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id,
                    kind_id,
                    client_key: second.client_key.clone(),
                    fields: name_field_patch("second-persisted-meaning"),
                }))),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let committed = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("both creates commit");
    let first_id = committed
        .created_entity(&first)
        .expect("first create reference resolves");
    let second_id = committed
        .created_entity(&second)
        .expect("second create reference resolves");
    assert_ne!(first_id, second_id);

    let records = runtime
        .read_truth()
        .project_historical_version(committed.version_id)
        .all_authoritative_entity_records();
    let first_record = records
        .iter()
        .find(|record| record.entity_id == first_id)
        .expect("first resolved identity exists in committed truth");
    let second_record = records
        .iter()
        .find(|record| record.entity_id == second_id)
        .expect("second resolved identity exists in committed truth");
    assert_eq!(
        read_entity_name(first_record),
        Some("first-persisted-meaning".to_owned())
    );
    assert_eq!(
        read_entity_name(second_record),
        Some("second-persisted-meaning".to_owned())
    );
}
