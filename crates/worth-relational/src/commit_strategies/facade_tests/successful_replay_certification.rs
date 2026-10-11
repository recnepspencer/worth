use super::native_strategy_fixtures::*;

#[test]
fn replay_commit_certifies_strategy_surface_for_strategy_bearing_commit() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = strategy_descriptor();
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .commit_strategy_executor(CommitStrategyExecutionRegistration::new(
            &descriptor,
            PlanningExecutor,
        ))
        .build();
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &crate::commit_strategies::data::NativeStrategyCommitRequest::from_native_canonical_bytes(
                CommitStrategySemanticName::new("strategy.intent.reconcile"),
                b"fixture-input".to_vec(),
                StrategyCallerProvenance {
                    request_origin: StrategyRequestOrigin::Test,
                    actor_identity: None,
                    correlation_id: None,
                },
            ),
        )
        .expect("canonical strategy request");
    let snapshot: SnapshotHandle = runtime.visibility_authority().snapshot();
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot, request_execution)
        .expect("strategy executes against committed basis");
    let commit = {
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime, None);
        let lowered = authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered strategy plan");
        let validated = authority
            .validate_lowered_plan(&runtime, lowered)
            .expect("validated lowered strategy plan");
        authority
            .execute_validated_commit(request_execution, &runtime, validated)
            .expect("validated strategy commit executed")
    };

    let replay = runtime.replay_authority().replay_commit(
        RelationalReplayRequest {
            commit_id: commit.commit.commit_id,
            branch_id: commit.publication().envelope.branch_context.clone(),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
        },
        request_execution,
    );

    assert!(
        replay.failure.is_none(),
        "unexpected replay failure: {replay:?}"
    );
    assert!(replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    assert!(replay
        .mismatches
        .iter()
        .all(|mismatch| mismatch.surface != ReplayObservableSurface::Strategy));
}

#[test]
fn intent_reconciliation_strategy_commits_and_replays_end_to_end() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = IntentReconciliationStrategy::descriptor(CommitStrategyId(61));
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(
            CommitStrategyRegistration::new(descriptor.clone())
                .expect("intent strategy registration"),
        )
        .commit_strategy_executor(IntentReconciliationStrategy::execution_registration(
            &descriptor,
        ))
        .build();
    let entity = crate::tests::support::create_entity(request_execution, &runtime, "before");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &IntentReconciliationInput {
                entity_id: entity,
                desired_aspect_fields: crate::transactions::data::AspectFieldPatch::from_locator(
                    crate::transactions::data::planned_single_field_locator(
                        worth_foundational::facade::AspectKey::new("name")
                            .expect("valid test aspect key"),
                        FieldKey::new("name").expect("valid test field key"),
                    ),
                    worth_foundational::facade::AspectValue::String(
                        worth_foundational::facade::InternedString::Raw("after".to_string()),
                    ),
                ),
            }
            .into_native_canonical_request(StrategyCallerProvenance {
                request_origin: StrategyRequestOrigin::Test,
                actor_identity: None,
                correlation_id: None,
            })
            .expect("native canonical strategy request"),
        )
        .expect("canonical request");
    let snapshot: SnapshotHandle = runtime.visibility_authority().snapshot();
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot, request_execution)
        .expect("strategy executes against committed basis");
    let commit = {
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime, None);
        let lowered = authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered strategy plan");
        let validated = authority
            .validate_lowered_plan(&runtime, lowered)
            .expect("validated strategy plan");
        authority
            .execute_validated_commit(request_execution, &runtime, validated)
            .expect("validated strategy commit executed")
    };

    let current = runtime
        .read_truth()
        .read_version(request_execution, runtime.current_version_id());
    assert_eq!(
        read_entity_name(current.get_entity(entity).expect("committed entity")),
        Some("after".into())
    );
    assert_eq!(
        commit
            .publication()
            .strategy_artifacts
            .as_ref()
            .expect("strategy artifacts")
            .merge_descriptor()
            .semantic_name()
            .as_str(),
        IntentReconciliationStrategy::DEFAULT_SEMANTIC_NAME
    );

    let replay = runtime.replay_authority().replay_commit(
        RelationalReplayRequest {
            commit_id: commit.commit.commit_id,
            branch_id: commit.publication().envelope.branch_context.clone(),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
        },
        request_execution,
    );

    assert!(
        replay.failure.is_none(),
        "unexpected replay failure: {replay:?}"
    );
    assert!(replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    assert!(replay
        .mismatches
        .iter()
        .all(|mismatch| mismatch.surface != ReplayObservableSurface::Strategy));
}

#[test]
fn entity_replacement_reconciliation_strategy_commits_lineage_sensitive_replace_and_replays() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = EntityReplacementReconciliationStrategy::descriptor(CommitStrategyId(62));
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(
            CommitStrategyRegistration::new(descriptor.clone())
                .expect("replacement strategy registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(&descriptor),
        )
        .build();
    let entity = crate::tests::support::create_entity(request_execution, &runtime, "before");
    let original_lineage = runtime
        .lineage_access()
        .for_record(entity)
        .expect("original lineage")
        .lineage_id;
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &EntityReplacementReconciliationInput {
                entity_id: entity,
                replacement_client_key: "service-replacement".to_string(),
                desired_aspect_fields: strategy_name_and_replicas_patch("before", 3),
            }
            .into_native_canonical_request(StrategyCallerProvenance {
                request_origin: StrategyRequestOrigin::Test,
                actor_identity: None,
                correlation_id: None,
            })
            .expect("native canonical strategy request"),
        )
        .expect("canonical replacement request");
    let snapshot: SnapshotHandle = runtime.visibility_authority().snapshot();
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot, request_execution)
        .expect("replacement strategy executes against committed basis");
    let commit = {
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime, None);
        let lowered = authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered replacement strategy plan");
        let validated = authority
            .validate_lowered_plan(&runtime, lowered)
            .expect("validated replacement strategy plan");
        authority
            .execute_validated_commit(request_execution, &runtime, validated)
            .expect("validated replacement strategy commit executed")
    };
    let current = runtime
        .read_truth()
        .read_version(request_execution, runtime.current_version_id());
    let replacement_record = changed_entities(&commit)
        .into_iter()
        .find_map(|entity_id| current.get_entity(entity_id).cloned())
        .expect("replacement entity visible");
    let replacement_lineage = runtime
        .lineage_access()
        .for_record(replacement_record.entity_id)
        .expect("replacement lineage")
        .lineage_id;
    let strategy_artifacts = commit
        .publication()
        .strategy_artifacts
        .as_ref()
        .expect("replacement strategy artifacts");

    assert_ne!(original_lineage, replacement_lineage);
    assert_eq!(read_entity_name(&replacement_record), Some("before".into()));
    let expected_replicas_key =
        crate::storage::data::authoritative_aspect_value_field_comparison_key(
            &AspectValue::UInt64(3),
        );
    let replicas_locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new("replicas").expect("valid replicas aspect"),
        CanonicalFieldPath::single(FieldKey::new("replicas").expect("valid replicas field")),
    );
    assert_eq!(
        crate::visibility::materialization::read_records::entity_query_locus_comparison_key(
            &replacement_record,
            &replicas_locator
        ),
        Some(expected_replicas_key)
    );
    assert_eq!(
        strategy_artifacts
            .lowering_summary()
            .normalized_client_key_count(),
        1
    );
    assert_eq!(
        strategy_artifacts
            .lowering_summary()
            .lineage_transition_count(),
        1
    );
    assert!(commit
        .publication()
        .envelope
        .lineage_decision_log()
        .iter()
        .any(
            |decision| decision.kind == crate::lineage::data::LineageDecisionKind::ReplaceAccepted
        ));

    let replay = runtime.replay_authority().replay_commit(
        RelationalReplayRequest {
            commit_id: commit.commit.commit_id,
            branch_id: commit.publication().envelope.branch_context.clone(),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
        },
        request_execution,
    );

    assert!(
        replay.failure.is_none(),
        "unexpected replacement replay failure: {replay:?}"
    );
    assert!(replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    assert!(replay
        .mismatches
        .iter()
        .all(|mismatch| mismatch.surface != ReplayObservableSurface::Strategy));
}
