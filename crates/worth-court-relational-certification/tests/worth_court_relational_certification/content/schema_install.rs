//! Installation, rather than the first commit, refuses unbound top-level content.
use worth_foundational::facade::*;
use worth_relational::facade::{identity::KindId, schema::*};
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;

fn install(
    contract: AspectContract,
    relation: bool,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    let fixture = AspectSchemaFixture::default();
    let field = FieldKey::new("samples").unwrap();
    let binding = if relation {
        AspectBinding::RelationField { field }
    } else {
        AspectBinding::EntityField { field }
    };
    let declarations = KindAspectContractDeclarations::new(vec![DeclaredAspectContractBinding {
        binding,
        contract,
    }]);
    if relation {
        RelationalSchemaRegistry::new().register_relation_kind(RelationKindRegistration {
            kind_id: KindId(42),
            kind_name: "samples".into(),
            schema_id: fixture.schema_id,
            schema_version_id: fixture.schema_version_id,
            cross_context_policy: fixture.cross_context_policy,
            cascade_delete_policy: fixture.cascade_delete_policy,
            aspect_contract_declarations: declarations,
            relation_integrity: RelationIntegrityDeclarations::default(),
        })
    } else {
        RelationalSchemaRegistry::new().register_entity_kind(EntityKindRegistration {
            kind_id: KindId(42),
            kind_name: "samples".into(),
            schema_id: fixture.schema_id,
            schema_version_id: fixture.schema_version_id,
            aspect_contract_declarations: declarations,
        })
    }
}

#[test]
fn content_install_refuses_unbound_scalar_references_on_both_kind_doors() {
    let key = AspectKey::new("samples").unwrap();
    let identity = AspectIdentity(42);
    let revision = AspectContractRevision(1);
    let scalar = AspectContract::scalar(
        key.clone(),
        identity,
        revision,
        ScalarAspectType::ContentRef,
    );
    for relation in [false, true] {
        assert_eq!(
            install(scalar.clone(), relation).unwrap_err().class,
            SchemaRegistryErrorClass::InvalidAspectDeclaration {
                kind_id: KindId(42),
                detail: "content references require a format-bound struct field".into(),
            }
        );
        install(
            AspectContract::scalar(key.clone(), identity, revision, ScalarAspectType::Int64),
            relation,
        )
        .unwrap();
    }
    // This existing fixture actually installs two format-bound ContentRef fields.
    let (_, contract) = super::transaction::fixture_registry(
        std::sync::Arc::new(super::fixture::Codec {
            descriptor: ContentFormat::new("sample-grid", 1).unwrap(),
            alter: false,
        }),
        SchemaVersionId(1),
    );
    assert!(matches!(contract.shape(), AspectShape::Struct(_)));
}
