use crate::tests::support::*;

#[test]
fn savepoint_abandoned_work_never_appears_in_subscriber_cdc() {
    let runtime = runtime_with_test_schema();
    let anchor = create_entity_outcome(&runtime, "anchor");
    let anchor_entity = changed_entities(&anchor)[0];
    let checkpoint = checkpoint_for_schema_version(anchor.patch_position(), SchemaVersionId(1));

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(batch_create("surviving"))
        .expect("test staging stays within configured resource budgets");
    let savepoint = txn.create_savepoint().unwrap();
    txn.push_batch(batch_create("abandoned"))
        .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("abandoned-update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: anchor_entity,
                fields: single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    field_key("name"),
                    "abandoned-anchor",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let rollback = txn.rollback_to_savepoint(savepoint).unwrap();
    txn.push_batch(
        WorkerIntentBatch::new("survived-update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: anchor_entity,
                fields: single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    field_key("name"),
                    "survived-anchor",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(&runtime).unwrap();

    assert!(rollback.summary().has_discarded_entity_creation());

    assert_subscriber_stream_omits_detail(&runtime, checkpoint, "abandoned");

    let read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .unwrap();
    let names = read
        .entities()
        .iter()
        .filter_map(read_entity_name)
        .collect::<Vec<_>>();

    assert!(names.contains(&"surviving".into()));
    assert!(names.contains(&"survived-anchor".into()));
    assert!(!names.contains(&"abandoned".into()));
    assert!(!names.contains(&"abandoned-anchor".into()));
}
