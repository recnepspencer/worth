use super::fixtures::*;

#[test]
fn invariant_failure_artifact_preserves_specific_code_localization_and_proof_boundary() {
    let runtime = RelationIntegritySchemaFixture {
        relation_integrity: RelationIntegrityDeclarations::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![SymmetryContractDeclaration {
                contract_id: "paired_twin".into(),
                mode: SymmetryMode::PairedTwinRequired,
            }],
            Vec::new(),
        ),
        ..RelationIntegritySchemaFixture::default()
    }
    .build_runtime();
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
    let diagnostics = runtime.publication().diagnostics();
    let artifact = diagnostics
        .by_scope(DiagnosticsScope::Invariant)
        .into_iter()
        .find(|artifact| {
            artifact.kind == DiagnosticsArtifactKind::Failure
                && artifact
                    .entries
                    .iter()
                    .any(|entry| entry.code == DiagnosticCode::RelationSymmetryViolation)
        })
        .expect("invariant failure artifact");
    let entry = artifact
        .entries
        .iter()
        .find(|entry| entry.code == DiagnosticCode::RelationSymmetryViolation)
        .expect("relation symmetry failure entry");

    match error {
        TransactionCommitError::Conflict { error, .. } => {
            assert_eq!(error.code(), DiagnosticCode::RelationSymmetryViolation);
        }
        other => panic!("expected conflict, got {:?}", other),
    }
    assert_eq!(
        diagnostic_field(entry, "execution_point"),
        &RelationalDiagnosticValue::string("commit_boundary")
    );
    assert_eq!(
        diagnostic_field(entry, "failure_effect"),
        &RelationalDiagnosticValue::string("block_commit")
    );
    let violation = diagnostic_field(entry, "violation");
    assert_eq!(
        diagnostic_object_field(violation, "violation_kind"),
        &RelationalDiagnosticValue::string("relation_symmetry")
    );
    assert_eq!(
        diagnostic_object_field(violation, "contract_id"),
        &RelationalDiagnosticValue::ContractId(ContractId::new("paired_twin"))
    );
    assert_eq!(
        diagnostic_object_field(violation, "relation_kind_id"),
        &RelationalDiagnosticValue::KindId(KindId(2))
    );
    assert_eq!(
        diagnostic_object_field(violation, "source"),
        &existing_entity_reference_diagnostic_value(source)
    );
    assert_eq!(
        diagnostic_object_field(violation, "target"),
        &existing_entity_reference_diagnostic_value(target)
    );
    assert_eq!(
        diagnostic_object_field(violation, "mode"),
        &RelationalDiagnosticValue::string("paired_twin_required")
    );
    let proof_boundary = diagnostic_field(entry, "proof_boundary");
    assert_eq!(
        diagnostic_object_field(proof_boundary, "scope_class"),
        &RelationalDiagnosticValue::string("partition_scope")
    );
    assert_eq!(
        diagnostic_object_field(proof_boundary, "packet_count"),
        &RelationalDiagnosticValue::Unsigned(1)
    );
}

#[test]
fn invariant_diagnostics_trace_proof_boundary_for_relation_integrity_execution() {
    let runtime = RelationIntegritySchemaFixture {
        relation_integrity: RelationIntegrityDeclarations::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![SymmetryContractDeclaration {
                contract_id: "paired_twin".into(),
                mode: SymmetryMode::PairedTwinRequired,
            }],
            Vec::new(),
        ),
        ..RelationIntegritySchemaFixture::default()
    }
    .build_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("paired").push(MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("forward"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("paired-inverse").push(MutationIntent::Create(
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
    txn.commit(
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();

    let diagnostics = runtime.publication().diagnostics();
    let entry = diagnostics
        .by_scope(DiagnosticsScope::Invariant)
        .into_iter()
        .filter(|artifact| artifact.kind == DiagnosticsArtifactKind::DetailedTrace)
        .flat_map(|artifact| artifact.entries.iter())
        .find(|entry| {
            entry.code == DiagnosticCode::InvariantProofBoundaryObserved
                && diagnostic_field_optional(entry, "execution_point")
                    == Some(&RelationalDiagnosticValue::string("commit_boundary"))
                && diagnostic_field_optional(entry, "proof_boundary").is_some_and(|value| {
                    diagnostic_object_field(value, "packet_count")
                        == &RelationalDiagnosticValue::Unsigned(1)
                })
        })
        .expect("proof boundary trace entry");

    assert_eq!(
        diagnostic_field(entry, "execution_point"),
        &RelationalDiagnosticValue::string("commit_boundary")
    );
    let proof_boundary = diagnostic_field(entry, "proof_boundary");
    assert_eq!(
        diagnostic_object_field(proof_boundary, "scope_class"),
        &RelationalDiagnosticValue::string("partition_scope")
    );
    assert_eq!(
        diagnostic_object_field(proof_boundary, "packet_count"),
        &RelationalDiagnosticValue::Unsigned(1)
    );
    assert_eq!(
        diagnostic_object_field(proof_boundary, "touched_partition_count"),
        &RelationalDiagnosticValue::Unsigned(1)
    );
}

#[test]
fn collect_all_invariant_failures_emits_multiple_relation_integrity_entries_for_one_commit() {
    let diagnostics = RelationalDiagnosticsProfile {
        collect_all_invariant_failures: true,
        ..RelationalDiagnosticsProfile::default()
    };
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(
            RelationIntegritySchemaFixture {
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: "no_self".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::AllowExplicit,
                    }],
                    Vec::new(),
                    Vec::new(),
                    vec![SymmetryContractDeclaration {
                        contract_id: "paired_twin".into(),
                        mode: SymmetryMode::InverseProhibited,
                    }],
                    Vec::new(),
                ),
                ..RelationIntegritySchemaFixture::default()
            }
            .build_registry(),
        )
        .diagnostics(diagnostics)
        .build();
    let source = create_entity(&runtime, "source");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("self-edge").push(MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("self-edge"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(source),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let _error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();

    let diagnostics = runtime.publication().diagnostics();
    let failure_artifact = diagnostics
        .by_scope(DiagnosticsScope::Invariant)
        .into_iter()
        .find(|artifact| artifact.kind == DiagnosticsArtifactKind::Failure)
        .expect("collect-all invariant failure artifact");

    assert!(failure_artifact
        .entries
        .iter()
        .any(|entry| entry.code == DiagnosticCode::RelationEndpointKindViolation));
    assert!(failure_artifact
        .entries
        .iter()
        .any(|entry| entry.code == DiagnosticCode::RelationSymmetryViolation));
}
