use super::*;

#[test]
fn replay_contract_reports_schema_lineage_drift_at_summary_layer_when_digest_is_unavailable() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(execution, &runtime, "a");

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
    let outcome = txn.commit(execution, &runtime).unwrap();

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
        .replay_commit(
            RelationalReplayRequest {
                commit_id: outcome.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();

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
