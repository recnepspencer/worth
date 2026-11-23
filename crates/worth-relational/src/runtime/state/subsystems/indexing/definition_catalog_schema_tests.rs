//! Routing distinguishes real schema meanings at the same version.
use super::*;

#[test]
fn schema_meanings_with_one_version_keep_distinct_kind_routes() {
    use crate::identity::data::KindId;
    use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexKind};
    use crate::tests::support::{aspect_field_locator, aspect_key, field_key};
    let old = crate::tests::support::declared_aspect_schema_registry(
        crate::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let mut added = old.entity_kinds.get(&KindId(1)).unwrap().clone();
    added.kind_id = KindId(91);
    added.kind_name = "new-index-covered-kind".into();
    added
        .aspect_contract_declarations
        .identity_declarations
        .clear();
    let new = old.clone().register_entity_kind(added).unwrap();
    assert_eq!(
        old.primary_schema_version_id(),
        new.primary_schema_version_id()
    );
    let indexes = IndexingSubsystem::default();
    let definition = indexes
        .register_definition(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "schema-qualified-routing".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .unwrap();
    indexes.install_routing(
        old.authority_digest_bytes(),
        &crate::schema::lower_aspect_plans(&old),
    );
    indexes.install_routing(
        new.authority_digest_bytes(),
        &crate::schema::lower_aspect_plans(&new),
    );
    let catalog = indexes.definition_read_binding().snapshot();
    let old_routes = catalog.routing(old.authority_digest_bytes()).unwrap();
    let new_routes = catalog.routing(new.authority_digest_bytes()).unwrap();
    assert!(old_routes.entities(KindId(91)).is_empty());
    assert_eq!(new_routes.entities(KindId(91)), &[definition.index_id]);
    assert_eq!(
        old_routes.entities(KindId(1)),
        new_routes.entities(KindId(1))
    );
}
