//! Generic Query-shaped declarations; no proprietary codecs or values.
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::identity::data::KindId;
use crate::schema::data::*;
use worth_foundational::facade::{
    aspects, AspectBinding, AspectIdentity, AspectKey, FieldKey, ScalarAspectType,
    StructFieldBuilder,
};

pub(super) fn binding(
    name: &str,
    identity: u64,
    required: &[&str],
    optional: &[&str],
) -> DeclaredAspectContractBinding {
    let mut fields = StructFieldBuilder::default();
    for field in required {
        fields = fields.required(*field, ScalarAspectType::String);
    }
    for field in optional {
        fields = fields.optional(*field, ScalarAspectType::String);
    }
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField {
            field: FieldKey::new(name).unwrap(),
        },
        contract: aspects()
            .contract()
            .for_key(AspectKey::new(name).unwrap())
            .identified_by(AspectIdentity(identity))
            .at_revision(aspects().vocabulary().revision(1))
            .struct_aspect(fields.finish().unwrap()),
    }
}
pub(super) fn facts() -> DeclaredAspectContractBinding {
    binding("facts", 3002, &["a", "b", "c"], &["note"])
}
pub(super) fn registry() -> RelationalSchemaRegistry {
    let mut registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".into(),
            schema_id: SchemaId("test".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|schema| {
            schema.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".into(),
                schema_id: SchemaId("test".into()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .unwrap();
    for (kind, declarations) in [
        (
            3,
            vec![
                binding("identity", 3001, &["key"], &[]),
                facts(),
                binding("attachment", 3003, &[], &["reference"]),
            ],
        ),
        (4, vec![binding("optional", 4001, &[], &["value"])]),
        (
            5,
            vec![
                binding("geometry", 5001, &["value"], &[]),
                binding("nominal", 5002, &[], &["value"]),
            ],
        ),
        (6, vec![binding("value", 6001, &["value"], &[])]),
    ] {
        registry = registry
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(kind),
                kind_name: format!("generic-{kind}"),
                schema_id: SchemaId("test".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(declarations),
            })
            .unwrap();
    }
    registry
}
