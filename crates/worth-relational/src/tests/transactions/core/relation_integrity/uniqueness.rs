use crate::tests::support::*;

#[test]
fn relation_integrity_commit_boundary_rejects_duplicate_normalized_symmetric_edge() {
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
                    Vec::new(),
                    Vec::new(),
                    vec![crate::schema::data::UniquenessContractDeclaration {
                        contract_id: "uniq_norm".into(),
                        scope: crate::schema::data::UniquenessScope::NormalizedSymmetricEdge,
                    }],
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");

    create_relation(&runtime, source, target, "forward");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("duplicate-normalized").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("reverse"),
                source: crate::transactions::data::EntityReference::Existing(target),
                target: crate::transactions::data::EntityReference::Existing(source),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
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
            assert_eq!(error.code(), DiagnosticCode::RelationUniquenessViolation);
        }
        other => panic!("expected conflict, got {:?}", other),
    }
}
