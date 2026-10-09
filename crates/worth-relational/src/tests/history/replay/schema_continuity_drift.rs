use super::*;
use worth_execution::ExecutionAllocationPolicy as Allocation;

#[test]
fn replay_contract_reports_schema_continuation_descriptor_drift_when_envelope_is_tampered() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::default()
        }
        .build_registry(),
    );

    let proposed_transition = ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_string()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".to_string()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                SchemaVersionId(2),
                Some(KindId(1)),
                "tag",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )],
    };
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    proposed_transition,
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(batch_create("b"), Allocation::SystemAllocation)
        .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(&runtime, Allocation::SystemAllocation).unwrap();

    assert!(runtime.history_authority().tamper_commit_envelope_for_test(
        outcome.commit.commit_id,
        |envelope| {
            if let Some(descriptor) = envelope.schema_continuation_descriptor.as_mut() {
                descriptor.normalized_boundary_count += 1;
            }
        }
    ));

    let replay = runtime
        .replay_authority()
        .replay_commit(RelationalReplayRequest {
            commit_id: outcome.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
        });

    assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch));
    assert!(
        replay.mismatches.iter().any(|mismatch| {
            mismatch.class == ReplayMismatchClass::SchemaContinuationDescriptorDrift
                && mismatch.surface == ReplayObservableSurface::History
                && mismatch.verification_layer
                    == crate::facade::replay::ReplayVerificationLayer::DigestParity
        }),
        "{:?}",
        replay.mismatches
    );
}

#[test]
fn replay_contract_audit_mode_confirms_schema_continuation_descriptor_drift_at_deep_layer() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::default()
        }
        .build_registry(),
    );

    let proposed_transition = ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_string()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".to_string()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                SchemaVersionId(2),
                Some(KindId(1)),
                "tag",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )],
    };
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    proposed_transition,
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(batch_create("b"), Allocation::SystemAllocation)
        .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(&runtime, Allocation::SystemAllocation).unwrap();

    assert!(runtime.history_authority().tamper_commit_envelope_for_test(
        outcome.commit.commit_id,
        |envelope| {
            if let Some(descriptor) = envelope.schema_continuation_descriptor.as_mut() {
                descriptor.normalized_boundary_count += 1;
            }
        }
    ));

    let replay = runtime
        .replay_authority()
        .replay_commit(RelationalReplayRequest {
            commit_id: outcome.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
        });

    assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch));
    assert!(
        replay.mismatches.iter().any(|mismatch| {
            mismatch.class == ReplayMismatchClass::SchemaContinuationDescriptorDrift
                && mismatch.surface == ReplayObservableSurface::History
                && mismatch.verification_layer
                    == crate::facade::replay::ReplayVerificationLayer::DeepArtifactParity
        }),
        "{:?}",
        replay.mismatches
    );
}

#[test]
fn replay_certification_audit_drift_is_explained_and_counted() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::default()
        }
        .build_registry(),
    );

    let proposed_transition = ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_string()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".to_string()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                SchemaVersionId(2),
                Some(KindId(1)),
                "tag",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )],
    };
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    proposed_transition,
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(batch_create("b"), Allocation::SystemAllocation)
        .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(&runtime, Allocation::SystemAllocation).unwrap();

    assert!(runtime.history_authority().tamper_commit_envelope_for_test(
        outcome.commit.commit_id,
        |envelope| {
            if let Some(descriptor) = envelope.schema_continuation_descriptor.as_mut() {
                descriptor.normalized_boundary_count += 1;
            }
        }
    ));

    runtime.performance_access().reset_counters();
    let replay = runtime
        .replay_authority()
        .replay_commit(RelationalReplayRequest {
            commit_id: outcome.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
        });

    assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch));
    let diagnostics = runtime.publication().diagnostics();
    let authority_continuity_entry = diagnostics
        .by_scope(DiagnosticsScope::Replay)
        .into_iter()
        .flat_map(|artifact| artifact.entries.iter())
        .find(|entry| {
            entry.code == DiagnosticCode::InvariantViolation
                && diagnostic_field_optional(entry, "verification_mode")
                    == Some(&RelationalDiagnosticValue::string(
                        "AuditRecoveryVerification",
                    ))
                && diagnostic_field_optional(entry, "mismatch_count")
                    == Some(&RelationalDiagnosticValue::Unsigned(
                        replay.mismatches.len() as u64,
                    ))
        })
        .expect("replay certification diagnostic");
    assert_eq!(
        diagnostic_field(authority_continuity_entry, "verification_mode"),
        &RelationalDiagnosticValue::string("AuditRecoveryVerification")
    );
    assert!(diagnostic_array_contains_string(
        authority_continuity_entry,
        "mismatch_verification_layers",
        "DeepArtifactParity",
    ));
    assert!(diagnostic_array_contains_string(
        authority_continuity_entry,
        "mismatch_classes",
        "SchemaContinuationDescriptorDrift",
    ));
    let counters = runtime.performance_access().counters();
    assert!(counters.replay_deep_artifact_parity_checks >= 1);
    assert_eq!(counters.replay_summary_parity_checks, 0);
}

#[test]
fn replay_contract_reports_schema_lineage_drift_at_summary_layer_when_digest_is_unavailable() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::default()
        }
        .build_registry(),
    );

    let proposed_transition = ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_string()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".to_string()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                SchemaVersionId(2),
                Some(KindId(1)),
                "tag",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )],
    };
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    proposed_transition,
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(batch_create("b"), Allocation::SystemAllocation)
        .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(&runtime, Allocation::SystemAllocation).unwrap();

    assert!(runtime.history_authority().tamper_commit_envelope_for_test(
        outcome.commit.commit_id,
        |envelope| {
            if let Some(descriptor) = envelope.schema_reconciliation_descriptor.as_mut() {
                descriptor
                    .resulting_lineage
                    .parent_schema_version_ids
                    .push(SchemaVersionId(999));
            }
            if let Some(transition) = envelope.schema_transition.as_mut() {
                transition
                    .reconciliation_descriptor
                    .resulting_lineage
                    .parent_schema_version_ids
                    .push(SchemaVersionId(999));
            }
        }
    ));

    let replay = runtime
        .replay_authority()
        .replay_commit(RelationalReplayRequest {
            commit_id: outcome.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
        });

    assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch));
    assert!(
        replay.mismatches.iter().any(|mismatch| {
            mismatch.class == ReplayMismatchClass::SchemaLineageDrift
                && mismatch.surface == ReplayObservableSurface::History
                && mismatch.verification_layer
                    == crate::facade::replay::ReplayVerificationLayer::SummaryParity
        }),
        "{:?}",
        replay.mismatches
    );
}
