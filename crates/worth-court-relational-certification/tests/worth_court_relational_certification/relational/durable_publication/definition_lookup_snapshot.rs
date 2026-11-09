//! Detached definition DATA keeps its original owner-issued numeric identity.
use worth_relational::facade::indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use worth_testkit_relational::{
    aspect_storage::{aspect_field_locator, aspect_key, field_key},
    durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};
#[test]
fn lookup_snapshot_is_detached_from_subsequent_owner_registration() {
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let expected = DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "snapshot-index".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    };
    let installed = runtime
        .index_authority()
        .register(expected.clone())
        .expect("index fixture has identity capacity");
    let snapshot = runtime.index_access().definition_lookup_snapshot();
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            name: "later-index".to_owned(),
            ..expected.clone()
        })
        .expect("index fixture has identity capacity");

    assert_eq!(snapshot.definition_count(), 1);
    assert_eq!(snapshot.candidate_count_for_name("snapshot-index"), 1);
    assert_eq!(snapshot.candidate_count_for_name("later-index"), 0);
    assert_eq!(
        snapshot.matching_definition(&expected).unwrap().index_id,
        installed.index_id
    );
}
