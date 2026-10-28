use worth_foundational::{
    aspects, AspectFieldLocator, AspectIdentity, CanonicalFieldPath, FieldKey, LocatorAuthority,
    ScalarAspectType,
};

use super::*;

pub(crate) fn test_schema_registry() -> RelationalSchemaRegistry {
    declared_aspect_schema_registry(CascadeDeletePolicy::CascadeDeleteRelations)
}

pub(crate) fn aspect_key(name: &str) -> AspectKey {
    AspectKey::new(name).unwrap()
}

pub(crate) fn field_key(name: &str) -> FieldKey {
    FieldKey::new(name).expect("test field names must be foundational field keys")
}

pub(crate) fn aspect_field_locator(aspect_key: AspectKey, field: FieldKey) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect_key,
        CanonicalFieldPath::single(field),
    )
}

pub(crate) fn entity_field_aspect(
    aspect_key: AspectKey,
    field: FieldKey,
) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField { field },
        contract: scalar_string_contract(aspect_key),
    }
}

pub(crate) fn entity_u64_field_aspect(
    aspect_key: AspectKey,
    field: FieldKey,
) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField { field },
        contract: scalar_u64_contract(aspect_key),
    }
}

pub(crate) fn entity_i64_field_aspect(
    aspect_key: AspectKey,
    field: FieldKey,
) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField { field },
        contract: scalar_i64_contract(aspect_key),
    }
}

pub(crate) fn entity_bool_field_aspect(
    aspect_key: AspectKey,
    field: FieldKey,
) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField { field },
        contract: scalar_bool_contract(aspect_key),
    }
}

pub(crate) fn entity_summary_struct_aspect(
    aspect_key: AspectKey,
    field: FieldKey,
) -> DeclaredAspectContractBinding {
    let shape = aspects()
        .struct_fields()
        .required("title", ScalarAspectType::String)
        .optional("status", ScalarAspectType::String)
        .finish()
        .expect("valid entity summary struct aspect shape");
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField { field },
        contract: aspects()
            .contract()
            .for_key(aspect_key.clone())
            .identified_by(AspectIdentity(test_contract_identity(&aspect_key)))
            .at_revision(aspects().vocabulary().revision(1))
            .struct_aspect(shape),
    }
}

pub(crate) fn relation_field_aspect(
    aspect_key: AspectKey,
    field: FieldKey,
) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::RelationField { field },
        contract: scalar_string_contract(aspect_key),
    }
}

pub(crate) fn lifecycle_aspect() -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::LifecycleTransition,
        contract: scalar_string_contract(aspect_key("lifecycle")),
    }
}

pub(crate) fn relation_source_aspect() -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::RelationSourceEndpoint,
        contract: entity_reference_contract(aspect_key("source")),
    }
}

pub(crate) fn relation_target_aspect() -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::RelationTargetEndpoint,
        contract: entity_reference_contract(aspect_key("target")),
    }
}

fn scalar_string_contract(aspect_key: AspectKey) -> worth_foundational::AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key.clone())
        .identified_by(AspectIdentity(test_contract_identity(&aspect_key)))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::String)
}

fn scalar_bool_contract(aspect_key: AspectKey) -> worth_foundational::AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key.clone())
        .identified_by(AspectIdentity(test_contract_identity(&aspect_key)))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::Bool)
}

fn scalar_u64_contract(aspect_key: AspectKey) -> worth_foundational::AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key.clone())
        .identified_by(AspectIdentity(test_contract_identity(&aspect_key)))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::UInt64)
}

fn scalar_i64_contract(aspect_key: AspectKey) -> worth_foundational::AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key.clone())
        .identified_by(AspectIdentity(test_contract_identity(&aspect_key)))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::Int64)
}

fn entity_reference_contract(aspect_key: AspectKey) -> worth_foundational::AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key.clone())
        .identified_by(AspectIdentity(test_contract_identity(&aspect_key)))
        .at_revision(aspects().vocabulary().revision(1))
        .reference_entity()
}

fn test_contract_identity(aspect_key: &AspectKey) -> u64 {
    let mut hash = 14695981039346656037_u64;
    for byte in aspect_key.as_str().as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211_u64);
    }
    hash
}

#[derive(Debug, Clone)]
pub(crate) struct AspectSchemaFixture {
    pub entity_kind_id: KindId,
    pub relation_kind_id: KindId,
    pub entity_kind_name: String,
    pub relation_kind_name: String,
    pub schema_id: SchemaId,
    pub schema_version_id: SchemaVersionId,
    pub cross_context_policy: CrossContextPolicy,
    pub cascade_delete_policy: CascadeDeletePolicy,
    pub entity_aspects: Vec<DeclaredAspectContractBinding>,
    pub relation_aspects: Vec<DeclaredAspectContractBinding>,
}

impl Default for AspectSchemaFixture {
    fn default() -> Self {
        Self {
            entity_kind_id: KindId(1),
            relation_kind_id: KindId(2),
            entity_kind_name: "test.entity".to_string(),
            relation_kind_name: "test.relation".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            entity_aspects: Vec::new(),
            relation_aspects: Vec::new(),
        }
    }
}

impl AspectSchemaFixture {
    pub(crate) fn with_default_declared_aspects(
        cascade_delete_policy: CascadeDeletePolicy,
    ) -> Self {
        Self {
            cascade_delete_policy,
            entity_aspects: vec![
                entity_field_aspect(aspect_key("name"), field_key("name")),
                lifecycle_aspect(),
            ],
            relation_aspects: vec![
                relation_field_aspect(aspect_key("label"), field_key("label")),
                lifecycle_aspect(),
                relation_source_aspect(),
                relation_target_aspect(),
            ],
            ..Self::default()
        }
    }

    pub(crate) fn build_registry(&self) -> RelationalSchemaRegistry {
        RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: self.entity_kind_id,
                kind_name: self.entity_kind_name.clone(),
                schema_id: self.schema_id.clone(),
                schema_version_id: self.schema_version_id,
                aspect_contract_declarations: KindAspectContractDeclarations::new(
                    self.entity_aspects.clone(),
                ),
            })
            .and_then(|registry| {
                registry.register_relation_kind(RelationKindRegistration {
                    kind_id: self.relation_kind_id,
                    kind_name: self.relation_kind_name.clone(),
                    schema_id: self.schema_id.clone(),
                    schema_version_id: self.schema_version_id,
                    cross_context_policy: self.cross_context_policy,
                    cascade_delete_policy: self.cascade_delete_policy,
                    aspect_contract_declarations: KindAspectContractDeclarations::new(
                        self.relation_aspects.clone(),
                    ),
                    relation_integrity: RelationIntegrityDeclarations::default(),
                })
            })
            .unwrap()
    }

    pub(crate) fn build_runtime(&self) -> RelationalRuntime {
        RelationalRuntimeApi::builder()
            .schema_registry(self.build_registry())
            .build()
            .expect("valid native runtime configuration")
    }
}

pub(crate) fn runtime_with_declared_aspect_schema(
    cascade_delete_policy: CascadeDeletePolicy,
) -> RelationalRuntime {
    AspectSchemaFixture::with_default_declared_aspects(cascade_delete_policy).build_runtime()
}

pub(crate) fn declared_aspect_schema_registry(
    cascade_delete_policy: CascadeDeletePolicy,
) -> RelationalSchemaRegistry {
    AspectSchemaFixture::with_default_declared_aspects(cascade_delete_policy).build_registry()
}
