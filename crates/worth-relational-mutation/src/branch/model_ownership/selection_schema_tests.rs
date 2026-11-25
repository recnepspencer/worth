//! Pure schema-admission policy: registry commitments, with no runtime roots.
use super::require_same_schema;
use crate::contracts::branch::model_ownership::read_denial::RelationalModelOwnershipDenial as Denial;
use crate::contracts::config::data::policies::{CascadeDeletePolicy, CrossContextPolicy};
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::schema::data::aspect_contracts::declared_binding::DeclaredAspectContractBinding;
use crate::contracts::schema::data::aspect_contracts::kind_declarations::KindAspectContractDeclarations;
use crate::contracts::schema::data::{
    EntityKindRegistration, RelationKindRegistration, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_foundational::facade::{
    AspectBinding, AspectContract, AspectContractRevision, AspectIdentity, AspectKey, FieldKey,
    ScalarAspectType,
};

fn registry(
    version: SchemaVersionId,
    entity_aspects: KindAspectContractDeclarations,
) -> RelationalSchemaRegistry {
    let schema_id = SchemaId("ownership".into());
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "document".into(),
            schema_id: schema_id.clone(),
            schema_version_id: version,
            aspect_contract_declarations: entity_aspects,
        })
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "member".into(),
            schema_id,
            schema_version_id: version,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: Default::default(),
            relation_integrity: Default::default(),
        })
        .unwrap()
}

#[test]
fn entity_boundary_refuses_changed_schema_meaning_before_structural_selection() {
    let original = registry(SchemaVersionId(1), Default::default());
    let current = original.authority_digest_bytes();
    let same = registry(SchemaVersionId(1), Default::default());
    assert!(require_same_schema(current, same.authority_digest_bytes()).is_ok());

    let entity_aspects = KindAspectContractDeclarations::new(vec![DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField {
            field: FieldKey::new("tag").unwrap(),
        },
        contract: AspectContract::scalar(
            AspectKey::new("tag").unwrap(),
            AspectIdentity(1),
            AspectContractRevision(1),
            ScalarAspectType::String,
        ),
    }]);
    for changed in [
        registry(SchemaVersionId(1), entity_aspects),
        registry(SchemaVersionId(2), Default::default()),
    ] {
        // Kind ids/counts alone are insufficient: neither alteration changes
        // the entity/relation identity sets. Use actual recorded schema meaning.
        assert_eq!(
            original
                .authority_snapshot()
                .entity_kinds
                .iter()
                .map(|kind| kind.kind_id)
                .collect::<Vec<_>>(),
            changed
                .authority_snapshot()
                .entity_kinds
                .iter()
                .map(|kind| kind.kind_id)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            original
                .authority_snapshot()
                .relation_kinds
                .iter()
                .map(|kind| kind.kind_id)
                .collect::<Vec<_>>(),
            changed
                .authority_snapshot()
                .relation_kinds
                .iter()
                .map(|kind| kind.kind_id)
                .collect::<Vec<_>>()
        );
        assert!(matches!(
            require_same_schema(current, changed.authority_digest_bytes()),
            Err(Denial::SchemaChanged)
        ));
    }
}
