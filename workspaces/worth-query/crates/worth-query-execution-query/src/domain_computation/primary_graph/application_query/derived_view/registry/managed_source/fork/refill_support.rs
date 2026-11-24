//! Pure schema and mutation-batch builders for expired pending-slot cold-refill tests.
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    identity::*,
    runtime::ManagedIndexSourceProgram,
    schema::*,
    symbols::ClientKey,
    transactions::*,
};

pub(super) fn refill_schema() -> (RelationalSchemaRegistry, ManagedIndexSourceProgram) {
    let sid = SchemaId("cold-refill-expired-slot-court".into());
    let mut reg = RelationalSchemaRegistry::new();
    for (kind, name) in [(1, "directory"), (4, "item")] {
        reg = reg
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(kind),
                kind_name: name.into(),
                schema_id: sid.clone(),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: Default::default(),
            })
            .unwrap();
    }
    reg = reg
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "contains".into(),
            schema_id: sid.clone(),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::Forbid,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: Default::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![EndpointKindContractDeclaration {
                    contract_id: "contains-endpoints".into(),
                    allowed_source_kinds: vec![KindId(1)],
                    allowed_target_kinds: vec![KindId(4)],
                    self_edges_allowed: false,
                    cross_context_policy: CrossContextPolicy::Forbid,
                }],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
        })
        .unwrap();
    let dr_id = DerivedRelationIdentity {
        schema: sid,
        version: SchemaVersionId(1),
        name: "dir-items".into(),
    };
    reg = reg
        .register_derived_relations(vec![DerivedRelationDefinitionParts {
            identity: dr_id.clone(),
            from: KindId(1).into(),
            to: KindId(4).into(),
            nodes: vec![DerivedRelationNode::Stored {
                relation: KindId(2),
                from: KindId(1),
                to: KindId(4),
            }],
            laws: None,
        }])
        .unwrap();
    let handle = reg.derived_relation(&dr_id).unwrap();
    let program = reg
        .bind_managed_index_source(&handle, vec![], vec![])
        .unwrap();
    (reg, program)
}

pub(super) fn seed_batch() -> (CreatedEntityRef, CreatedEntityRef, WorkerIntentBatch) {
    let ca = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw("A"),
    };
    let cx = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(4),
        client_key: ClientKey::raw("x"),
    };
    let batch = WorkerIntentBatch::new("seed")
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw("A"),
            fields: Default::default(),
        })))
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(4),
            client_key: ClientKey::raw("x"),
            fields: Default::default(),
        })))
        .push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("A-x"),
                source: EntityReference::Created(ca.clone()),
                target: EntityReference::Created(cx.clone()),
                fields: Default::default(),
            },
        )));
    (ca, cx, batch)
}

pub(super) fn collect_scope_roots(
    source: &worth_relational::facade::runtime::ManagedIndexSourcePreparation<'_>,
) -> Result<
    Vec<worth_relational::facade::runtime::PreparedManagedIndexSourceRoot>,
    worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial,
> {
    let mut roots = Vec::new();
    for scope in source.scopes() {
        let proposed = if scope.scope_exists()? {
            let rows = scope.proposed();
            let read = rows.acquire_all()?;
            let ids: Vec<EntityId> = rows
                .occurrences(&read)?
                .iter()
                .map(|o| o.entity())
                .collect();
            Some(std::sync::Arc::new(ids) as std::sync::Arc<dyn std::any::Any + Send + Sync>)
        } else {
            None
        };
        roots.push(scope.seal(None, proposed, Default::default()));
    }
    Ok(roots)
}

pub(super) fn anchor_schema() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "anchor".into(),
            schema_id: SchemaId("fork-lookup-court".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap()
}

pub(super) fn anchor_batch(label: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new(label).push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(label),
        fields: Default::default(),
    })))
}

pub(super) fn refused_publish_batch() -> WorkerIntentBatch {
    WorkerIntentBatch::new("refused").push(MutationIntent::Create(CreateIntent::Entity(
        EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(4),
            client_key: ClientKey::raw("y"),
            fields: Default::default(),
        },
    )))
}
