use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    identity::{KindId, PartitionId},
    runtime::RelationalRuntimeApi,
    schema::{
        AcyclicityContractDeclaration, AllowedCycleClass, ConnectivityMinimumContractDeclaration,
        ConnectivityMinimumEnforcement, DirectedTraversalKind, EntityKindRegistration,
        KindAspectContractDeclarations, RelationIntegrityDeclarations, RelationKindRegistration,
        RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, TransactionCommitError,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction,
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn commit_publication_stage_rejects_sources_without_required_connectivity() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(acyclicity_and_connectivity_registry())
        .build()
        .unwrap();
    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("node-a").push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("node-a"),
                fields: AspectFieldPatch::default(),
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn
        .commit(execution, &runtime)
        .expect_err("connectivity publication failure");
    match error {
        TransactionCommitError::Publication { error, .. } => {
            assert!(error.detail.contains("reachable_anchor"));
            assert!(error.detail.contains("at least 1 reachable target"));
        }
        other => panic!("expected publication error, got {other:?}"),
    }
}

fn acyclicity_and_connectivity_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "geom.node".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_entity_kind(EntityKindRegistration {
                kind_id: KindId(3),
                kind_name: "geom.anchor".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "geom.constraint".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default()
                    .with_acyclicity_contracts(vec![AcyclicityContractDeclaration {
                        contract_id: "no_cycles".into(),
                        traversal_direction: DirectedTraversalKind::SourceToTarget,
                        allowed_cycle_class: AllowedCycleClass::NoCycles,
                    }])
                    .with_connectivity_minimum_contracts(vec![
                        ConnectivityMinimumContractDeclaration {
                            contract_id: "reachable_anchor".into(),
                            source_kind_ids: vec![KindId(1)],
                            target_kind_ids: vec![KindId(3)],
                            minimum_reachable_targets: 1,
                            enforcement_boundary:
                                ConnectivityMinimumEnforcement::SnapshotPublication,
                        },
                    ]),
            })
        })
        .unwrap()
}
