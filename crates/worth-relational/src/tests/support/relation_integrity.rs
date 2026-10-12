use super::*;

#[derive(Debug, Clone)]
pub(crate) struct RelationIntegritySchemaFixture {
    pub relation_kind_id: KindId,
    pub relation_kind_name: String,
    pub cascade_delete_policy: CascadeDeletePolicy,
    pub relation_integrity: RelationIntegrityDeclarations,
}

impl Default for RelationIntegritySchemaFixture {
    fn default() -> Self {
        Self {
            relation_kind_id: KindId(2),
            relation_kind_name: "test.relation".to_string(),
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            relation_integrity: RelationIntegrityDeclarations::default(),
        }
    }
}

impl RelationIntegritySchemaFixture {
    pub(crate) fn build_registry(&self) -> RelationalSchemaRegistry {
        RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(1),
                kind_name: "test.entity".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
            .and_then(|registry| {
                registry.register_relation_kind(RelationKindRegistration {
                    kind_id: self.relation_kind_id,
                    kind_name: self.relation_kind_name.clone(),
                    schema_id: SchemaId("test".to_string()),
                    schema_version_id: SchemaVersionId(1),
                    cross_context_policy: CrossContextPolicy::AllowExplicit,
                    cascade_delete_policy: self.cascade_delete_policy,
                    aspect_contract_declarations: KindAspectContractDeclarations::default(),
                    relation_integrity: self.relation_integrity.clone(),
                })
            })
            .unwrap()
    }

    pub(crate) fn build_runtime(&self) -> RelationalRuntime {
        RelationalRuntimeApi::builder()
            .schema_registry(self.build_registry())
            .cascade_delete_policy(self.cascade_delete_policy)
            .build()
    }
}

pub(crate) fn endpoint_deletion_runtime(
    mode: crate::schema::data::EndpointDeletionIntegrityMode,
    cascade_delete_policy: CascadeDeletePolicy,
) -> RelationalRuntime {
    RelationIntegritySchemaFixture {
        cascade_delete_policy,
        relation_integrity: RelationIntegrityDeclarations::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![crate::schema::data::EndpointDeletionIntegrityDeclaration {
                contract_id: "endpoint_delete".into(),
                mode,
            }],
        ),
        ..RelationIntegritySchemaFixture::default()
    }
    .build_runtime()
}

pub(crate) fn create_endpoint_deletion_relation_fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    client_key: &str,
) -> (
    crate::facade::identity::EntityId,
    crate::facade::identity::EntityId,
    RelationId,
) {
    let source = create_entity(execution, runtime, "source");
    let target = create_entity(execution, runtime, "target");
    let relation = create_relation(execution, runtime, source, target, client_key);
    (source, target, relation)
}
