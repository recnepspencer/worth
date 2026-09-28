use crate::facade::TruthDeltaSurfaceKind;
use worth_foundational::facade::FieldKey;

use crate::relational_source::relational_test_support::{
    aspect_key, changed_entities, entity_summary_struct_aspect, field_key,
    single_string_aspect_field_patch, string_aspect_field_patch,
    test_owner_begin_transaction_for_main, AspectSchemaFixture,
};
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    CreateIntent, EntityMutationIntent, EntitySpec, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::support::bridge_envelopes_at_current_observation;

#[test]
fn a_committed_struct_field_patch_publishes_one_field_precise_target() {
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![entity_summary_struct_aspect(
            aspect_key("summary"),
            field_key("summary"),
        )],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let mut create = test_owner_begin_transaction_for_main(&runtime);
    create
        .push_batch(
            WorkerIntentBatch::new("create-summary").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("struct-patch"),
                    fields: string_aspect_field_patch([
                        (aspect_key("summary"), field_key("title"), "before"),
                        (aspect_key("summary"), field_key("status"), "open"),
                    ]),
                }),
            )),
        )
        .unwrap();
    let entity = changed_entities(&create.commit(&runtime).unwrap())[0];
    let mut update = test_owner_begin_transaction_for_main(&runtime);
    update
        .push_batch(
            WorkerIntentBatch::new("summary-field-patch").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: single_string_aspect_field_patch(
                        aspect_key("summary"),
                        field_key("title"),
                        "after",
                    ),
                }),
            )),
        )
        .unwrap();
    let commit_id = update.commit(&runtime).unwrap().commit.commit_id;

    let envelopes = bridge_envelopes_at_current_observation(runtime, [commit_id]);

    let items = envelopes[0].patch_body().canonical_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].surface_kind(), TruthDeltaSurfaceKind::EntityField);
    assert_eq!(
        items[0]
            .field_locator()
            .expect("field-precise bridge target")
            .field_path()
            .fields(),
        &[FieldKey::new("title").unwrap()]
    );
}
