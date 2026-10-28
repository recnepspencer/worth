//! Genuine registration, created keys and transaction stimuli for derived-source courts.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    identity::{KindId, PartitionId},
    mvcc::RelationalTransactionIntent,
    runtime::*,
    schema::*,
    symbols::ClientKey,
    transactions::*,
};
pub(super) fn registry() -> RelationalSchemaRegistry {
    registry_with_source(2)
}
pub(super) fn registry_with_source(first: u32) -> RelationalSchemaRegistry {
    let mut registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId::new(1),
            kind_name: "node".into(),
            schema_id: SchemaId("derived-audience".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    for kind in [2, 3] {
        registry = registry
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId::new(kind),
                kind_name: format!("source-{kind}"),
                schema_id: SchemaId("derived-audience".into()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::Forbid,
                cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
                aspect_contract_declarations: Default::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: format!("source-{kind}-endpoints").into(),
                        allowed_source_kinds: vec![KindId::new(1)],
                        allowed_target_kinds: vec![KindId::new(1)],
                        self_edges_allowed: true,
                        cross_context_policy: CrossContextPolicy::Forbid,
                    }],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                ),
            })
            .unwrap();
    }
    registry
        .register_derived_relations(
            [2, 3]
                .into_iter()
                .map(|kind| DerivedRelationDefinitionParts {
                    identity: identity(kind),
                    from: KindId::new(1).into(),
                    to: KindId::new(1).into(),
                    nodes: vec![DerivedRelationNode::Stored {
                        relation: KindId::new(if kind == 2 { first } else { kind }),
                        from: KindId::new(1),
                        to: KindId::new(1),
                    }],
                    laws: None,
                })
                .collect(),
        )
        .unwrap()
}
pub(super) fn identity(kind: u32) -> DerivedRelationIdentity {
    DerivedRelationIdentity {
        schema: SchemaId("derived-audience".into()),
        version: SchemaVersionId(1),
        name: format!("derived-{kind}"),
    }
}
pub(super) fn created(index: u32) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId::new(1),
        client_key: ClientKey::raw(format!("node-{index}")),
    }
}
pub(super) fn commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    batch: WorkerIntentBatch,
) -> CommitResult {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}
