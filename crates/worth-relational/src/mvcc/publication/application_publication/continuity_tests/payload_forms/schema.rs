//! Generic Query-shaped declarations; no proprietary codecs or values.
use super::*;
use crate::facade::runtime::RelationalRuntimeApi;
use crate::schema::data::*;
use worth_foundational::facade::{
    aspects, AspectBinding, AspectIdentity, ScalarAspectType, StructFieldBuilder,
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
            field: field_key(name),
        },
        contract: aspects()
            .contract()
            .for_key(aspect_key(name))
            .identified_by(AspectIdentity(identity))
            .at_revision(aspects().vocabulary().revision(1))
            .struct_aspect(fields.finish().unwrap()),
    }
}
pub(super) fn facts() -> DeclaredAspectContractBinding {
    binding("facts", 3002, &["a", "b", "c"], &["note"])
}
pub(super) fn runtime() -> crate::runtime::RelationalRuntime {
    let mut registry = AspectSchemaFixture::default().build_registry();
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
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
}
