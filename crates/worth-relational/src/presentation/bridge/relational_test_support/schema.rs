use worth_foundational::facade::{
    aspects, AspectBinding, AspectContract, AspectIdentity, AspectKey, FieldKey, ScalarAspectType,
};

use super::{aspect_key, field_key, ENTITY_KIND, RELATION_KIND};
use crate::facade::config::{CascadeDeletePolicy, CrossContextPolicy};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::schema::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationIntegrityDeclarations, RelationKindRegistration, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};

/// One entity kind and one relation kind, each declaring the given aspects.
#[derive(Debug, Clone)]
pub(crate) struct AspectSchemaFixture {
    pub(crate) cascade_delete_policy: CascadeDeletePolicy,
    pub(crate) entity_aspects: Vec<DeclaredAspectContractBinding>,
    pub(crate) relation_aspects: Vec<DeclaredAspectContractBinding>,
}

impl Default for AspectSchemaFixture {
    fn default() -> Self {
        Self {
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            entity_aspects: Vec::new(),
            relation_aspects: Vec::new(),
        }
    }
}

impl AspectSchemaFixture {
    /// Entities declare a `name` field and a lifecycle; relations declare a
    /// `label` field, a lifecycle, and both endpoints.
    pub(crate) fn with_default_declared_aspects(
        cascade_delete_policy: CascadeDeletePolicy,
    ) -> Self {
        Self {
            cascade_delete_policy,
            entity_aspects: vec![
                declared(
                    AspectBinding::EntityField {
                        field: field_key("name"),
                    },
                    scalar_string_contract(aspect_key("name")),
                ),
                lifecycle_aspect(),
            ],
            relation_aspects: vec![
                declared(
                    AspectBinding::RelationField {
                        field: field_key("label"),
                    },
                    scalar_string_contract(aspect_key("label")),
                ),
                lifecycle_aspect(),
                declared(
                    AspectBinding::RelationSourceEndpoint,
                    entity_reference_contract(aspect_key("source")),
                ),
                declared(
                    AspectBinding::RelationTargetEndpoint,
                    entity_reference_contract(aspect_key("target")),
                ),
            ],
        }
    }

    pub(crate) fn build_runtime(&self) -> RelationalRuntime {
        RelationalRuntimeApi::builder()
            .schema_registry(self.build_registry())
            .build()
    }

    fn build_registry(&self) -> RelationalSchemaRegistry {
        RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: ENTITY_KIND,
                kind_name: "test.entity".to_owned(),
                schema_id: SchemaId("test".to_owned()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(
                    self.entity_aspects.clone(),
                ),
            })
            .and_then(|registry| {
                registry.register_relation_kind(RelationKindRegistration {
                    kind_id: RELATION_KIND,
                    kind_name: "test.relation".to_owned(),
                    schema_id: SchemaId("test".to_owned()),
                    schema_version_id: SchemaVersionId(1),
                    cross_context_policy: CrossContextPolicy::AllowExplicit,
                    cascade_delete_policy: self.cascade_delete_policy,
                    aspect_contract_declarations: KindAspectContractDeclarations::new(
                        self.relation_aspects.clone(),
                    ),
                    relation_integrity: RelationIntegrityDeclarations::default(),
                })
            })
            .expect("the adapter test schema registers")
    }
}

/// A runtime built from [`AspectSchemaFixture::with_default_declared_aspects`].
pub(crate) fn runtime_with_declared_aspect_schema(
    cascade_delete_policy: CascadeDeletePolicy,
) -> RelationalRuntime {
    AspectSchemaFixture::with_default_declared_aspects(cascade_delete_policy).build_runtime()
}

/// An entity field holding a struct with a required `title` and an optional
/// `status`.
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
    declared(
        AspectBinding::EntityField { field },
        aspects()
            .contract()
            .for_key(aspect_key.clone())
            .identified_by(AspectIdentity(contract_identity(&aspect_key)))
            .at_revision(aspects().vocabulary().revision(1))
            .struct_aspect(shape),
    )
}

fn declared(binding: AspectBinding, contract: AspectContract) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding { binding, contract }
}

fn lifecycle_aspect() -> DeclaredAspectContractBinding {
    declared(
        AspectBinding::LifecycleTransition,
        scalar_string_contract(aspect_key("lifecycle")),
    )
}

fn scalar_string_contract(aspect_key: AspectKey) -> AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key.clone())
        .identified_by(AspectIdentity(contract_identity(&aspect_key)))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::String)
}

fn entity_reference_contract(aspect_key: AspectKey) -> AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key.clone())
        .identified_by(AspectIdentity(contract_identity(&aspect_key)))
        .at_revision(aspects().vocabulary().revision(1))
        .reference_entity()
}

/// A stable FNV-1a identity per aspect key, so every test runtime agrees.
fn contract_identity(aspect_key: &AspectKey) -> u64 {
    aspect_key
        .as_str()
        .bytes()
        .fold(14695981039346656037_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(1099511628211_u64)
        })
}
