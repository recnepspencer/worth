//! Routing distinguishes real schema meanings at the same version.
use super::*;

#[test]
fn schema_meanings_with_one_version_keep_distinct_kind_routes() {
    use crate::identity::data::KindId;
    use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexKind};
    use crate::schema::data::{
        DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
        RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    };
    use worth_foundational::facade::{
        aspects, AspectBinding, AspectIdentity, AspectKey, FieldKey, ScalarAspectType,
    };

    let aspect = AspectKey::new("name").unwrap();
    let field = FieldKey::new("name").unwrap();
    let contract = aspects()
        .contract()
        .for_key(aspect.clone())
        .identified_by(AspectIdentity(1))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::String);
    let old = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "base-index-covered-kind".into(),
            schema_id: SchemaId("index-schema".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                DeclaredAspectContractBinding {
                    binding: AspectBinding::EntityField {
                        field: field.clone(),
                    },
                    contract,
                },
            ]),
        })
        .unwrap();
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
                field_locator:
                    crate::transactions::data::aspect_field_patch::planned_single_field_locator(
                        aspect, field,
                    ),
            },
        })
        .unwrap();
    indexes.install_routing(
        old.authority_digest_bytes(),
        &crate::schema::aspect_plans::lower_aspect_plans(&old),
    );
    indexes.install_routing(
        new.authority_digest_bytes(),
        &crate::schema::aspect_plans::lower_aspect_plans(&new),
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
