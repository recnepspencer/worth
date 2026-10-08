use crate::tests::support::*;

#[test]
fn relation_integrity_commit_boundary_rejects_forbidden_self_edge() {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::new(
                    vec![crate::schema::data::EndpointKindContractDeclaration {
                        contract_id: "no_self".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::AllowExplicit,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let entity = create_entity(&runtime, "self");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("relation").push(MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("self"),
                source: crate::transactions::data::EntityReference::Existing(entity),
                target: crate::transactions::data::EntityReference::Existing(entity),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    match error {
        TransactionCommitError::Conflict { error, .. } => {
            assert_eq!(error.code(), DiagnosticCode::RelationEndpointKindViolation);
        }
        other => panic!("expected conflict, got {:?}", other),
    }
}
