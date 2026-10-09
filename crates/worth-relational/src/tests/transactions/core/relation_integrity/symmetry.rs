use crate::tests::support::*;

#[test]
fn relation_integrity_commit_boundary_requires_paired_inverse_edge() {
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
                    Vec::new(),
                    vec![crate::schema::data::SymmetryContractDeclaration {
                        contract_id: "paired_inverse".into(),
                        mode: crate::schema::data::SymmetryMode::PairedInverseRequired,
                    }],
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

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("one-way").push(MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("one-way"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
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
            assert_eq!(error.code(), DiagnosticCode::RelationSymmetryViolation);
            match error.class {
                crate::transactions::data::ConflictClass::InvariantViolation {
                    fields:
                        crate::validation::data::InvariantViolationFields::RelationSymmetry {
                            contract_id,
                            relation_kind_id,
                            source: actual_source,
                            target: actual_target,
                            mode,
                        },
                    ..
                } => {
                    assert_eq!(contract_id.as_str(), "paired_inverse");
                    assert_eq!(relation_kind_id, KindId(2));
                    assert_eq!(
                        actual_source,
                        crate::transactions::data::EntityReference::Existing(source)
                    );
                    assert_eq!(
                        actual_target,
                        crate::transactions::data::EntityReference::Existing(target)
                    );
                    assert_eq!(
                        mode,
                        crate::schema::data::SymmetryMode::PairedInverseRequired
                    );
                }
                other => panic!("expected typed symmetry invariant conflict, got {other:?}"),
            }
        }
        other => panic!("expected conflict, got {:?}", other),
    }
}

#[test]
fn relation_integrity_commit_boundary_requires_canonical_undirected_ordering() {
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
                    Vec::new(),
                    vec![crate::schema::data::SymmetryContractDeclaration {
                        contract_id: "canonical_undirected".into(),
                        mode: crate::schema::data::SymmetryMode::CanonicalUndirected,
                    }],
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let first = create_entity(&runtime, "first");
    let second = create_entity(&runtime, "second");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("reverse-undirected").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("reverse-undirected"),
                source: crate::transactions::data::EntityReference::Existing(second),
                target: crate::transactions::data::EntityReference::Existing(first),
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
            assert_eq!(error.code(), DiagnosticCode::RelationSymmetryViolation);
        }
        other => panic!("expected conflict, got {:?}", other),
    }
}

#[test]
fn relation_integrity_commit_boundary_prohibits_inverse_duplication() {
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
                    Vec::new(),
                    vec![crate::schema::data::SymmetryContractDeclaration {
                        contract_id: "inverse_prohibited".into(),
                        mode: crate::schema::data::SymmetryMode::InverseProhibited,
                    }],
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
        WorkerIntentBatch::new("inverse").push(MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("inverse"),
                source: crate::transactions::data::EntityReference::Existing(target),
                target: crate::transactions::data::EntityReference::Existing(source),
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
            assert_eq!(error.code(), DiagnosticCode::RelationSymmetryViolation);
        }
        other => panic!("expected conflict, got {:?}", other),
    }
}

#[test]
fn relation_integrity_commit_boundary_requires_paired_twin_edge() {
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
                    Vec::new(),
                    vec![crate::schema::data::SymmetryContractDeclaration {
                        contract_id: "paired_twin".into(),
                        mode: crate::schema::data::SymmetryMode::PairedTwinRequired,
                    }],
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

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("missing-twin").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("missing-twin"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
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
            assert_eq!(error.code(), DiagnosticCode::RelationSymmetryViolation);
        }
        other => panic!("expected conflict, got {:?}", other),
    }
}
