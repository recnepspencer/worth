use super::super::support::collect_subscriber_patches;
use super::fixtures::authoritative_patch_surface_contains;
use crate::tests::support::*;

#[test]
fn cdc_certification_savepoint_abandoned_work_never_leaks_into_stream_truth() {
    let runtime = runtime_with_test_schema();
    let left = create_entity_outcome(&runtime, "anchor-left");
    let right = create_entity_outcome(&runtime, "anchor-right");
    let left_entity = changed_entities(&left)[0];
    let right_entity = changed_entities(&right)[0];
    let checkpoint = checkpoint_for_schema_version(right.patch_position(), SchemaVersionId(1));

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(batch_create("surviving"))
        .expect("test staging stays within configured resource budgets");
    let savepoint = txn.create_savepoint().unwrap();
    txn.push_batch(batch_create("abandoned"))
        .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("abandoned-left").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: left_entity,
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "abandoned-left",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("abandoned-right").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: right_entity,
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "abandoned-right",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let rollback = txn.rollback_to_savepoint(savepoint).unwrap();
    txn.push_batch(
        WorkerIntentBatch::new("survived-left").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: left_entity,
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "survived-left",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("survived-right").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: right_entity,
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "survived-right",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(&runtime).unwrap();

    assert!(rollback.summary().has_discarded_entity_creation());

    let subscriber = collect_subscriber_patches(&runtime, checkpoint, 1);
    assert!(subscriber
        .iter()
        .flat_map(|patch| patch.authoritative_record_patches.iter())
        .all(|record| !authoritative_patch_surface_contains(record, "abandoned")));

    let patch_batch = runtime
        .publication()
        .read_patch_stream(PatchStreamRequest {
            after_position: Some(PatchStreamPosition(2)),
            max_commits: 32,
        })
        .unwrap();
    assert_eq!(subscriber, patch_batch.patches);

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
    assert!(names.contains(&"survived-left".into()));
    assert!(names.contains(&"survived-right".into()));
    assert!(!names.contains(&"abandoned".into()));
    assert!(!names.contains(&"abandoned-left".into()));
    assert!(!names.contains(&"abandoned-right".into()));
}
