//! Ordinary serial custody funds a complete 2,000-key replacement.
use super::*;
use crate::facade::transactions::{
    CreateIntent, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_execution::ExecutionRequest;
#[test]
fn two_thousand_key_commit_fits_the_ordinary_serial_request() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let mut batch = WorkerIntentBatch::new("bulk-index-setup");
    for row in 0..2000 {
        let name = format!("old-{row:04}");
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(&name),
                fields: name_field_patch(&name),
            },
        )));
    }
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let first = transaction.commit(&execution, &runtime).unwrap();
    let entities = changed_entities(&first);
    assert_eq!(entities.len(), 2000);
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "bulk-index.name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .index_id;
    let built = runtime.index_authority().build_for_commit(
        &execution,
        DerivedIndexBuildRequest {
            source_commit_id: first.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index],
        },
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    let mut batch = WorkerIntentBatch::new("bulk-index-update");
    for (row, entity_id) in entities.into_iter().enumerate() {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id,
                fields: name_field_patch(&format!("new-{row:04}")),
            },
        )));
    }
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let mut candidate = runtime
        .prepare_branch_transaction(&execution, transaction)
        .unwrap();
    let work = runtime
        .index_authority()
        .prepare_for_candidate(
            &mut candidate,
            &[index],
            Some(&first.snapshot),
            crate::facade::indexes::DerivedIndexMaintenanceBudget {
                maximum_work_units: 4_000_000,
                maximum_cold_record_slots: 0,
                maximum_derived_rows: 100_000,
            },
            &execution,
        )
        .unwrap();
    assert_eq!(work.entry_edits, 4000);
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("prepared candidate must publish")
    };
    let second = runtime.settle_performed_publication(performed).unwrap();
    let generation = runtime
        .index_access()
        .latest_generation(index, &BranchId("main".into()))
        .unwrap();
    assert_eq!(generation.source_commit_id, second.commit.commit_id);
    let crate::facade::indexes::DerivedIndexEntries::EntityField(entries) = &generation.entries
    else {
        panic!("field generation")
    };
    assert_eq!(entries.len(), 2000);

    release_test_commit_snapshot(&runtime, &second);
    release_test_commit_snapshot(&runtime, &first);
}
