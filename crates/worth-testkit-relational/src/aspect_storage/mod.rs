//! Public Native materialization of the one shared aspect-storage declaration.
pub mod definition;
pub mod publications;
pub use definition::{aspect_field_locator, aspect_key, field_key, optional_string_contract};
use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    identity::KindId,
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::*,
};

fn materialize((binding, contract): definition::DeclaredAspect) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding { binding, contract }
}

#[derive(Debug, Clone)]
pub struct AspectSchemaFixture {
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
        let names = definition::AspectSchemaNames::default();
        Self {
            entity_kind_id: KindId(names.entity_kind),
            relation_kind_id: KindId(names.relation_kind),
            entity_kind_name: names.entity_name.into(),
            relation_kind_name: names.relation_name.into(),
            schema_id: SchemaId(names.schema_name.into()),
            schema_version_id: SchemaVersionId(names.schema_revision),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            entity_aspects: vec![],
            relation_aspects: vec![],
        }
    }
}
impl AspectSchemaFixture {
    pub fn with_default_declared_aspects(cascade_delete_policy: CascadeDeletePolicy) -> Self {
        Self {
            cascade_delete_policy,
            entity_aspects: definition::default_entity_aspects()
                .into_iter()
                .map(materialize)
                .collect(),
            relation_aspects: definition::default_relation_aspects()
                .into_iter()
                .map(materialize)
                .collect(),
            ..Self::default()
        }
    }
    pub fn build_registry(&self) -> RelationalSchemaRegistry {
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
    pub fn build_runtime(&self) -> RelationalRuntime {
        RelationalRuntimeApi::builder()
            .schema_registry(self.build_registry())
            .build()
            .unwrap()
    }
}

macro_rules! materialized_binding {
    ($($name:ident),* $(,)?) => { $(
        pub fn $name(aspect: AspectKey, field: FieldKey) -> DeclaredAspectContractBinding {
            materialize(definition::$name(aspect, field))
        }
    )* };
}
materialized_binding!(
    entity_field_aspect,
    entity_u64_field_aspect,
    entity_i64_field_aspect,
    entity_bool_field_aspect,
    entity_summary_struct_aspect,
    relation_field_aspect
);
