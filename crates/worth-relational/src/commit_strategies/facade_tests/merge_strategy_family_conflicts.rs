use super::native_strategy_fixtures::*;

#[test]
fn merge_planning_classifies_different_strategy_families_as_strategy_intent_conflict() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let main_descriptor = strategy_descriptor_named(
        CommitStrategyId(41),
        "strategy.intent.reconcile",
        "strategy.intent",
        "reconcile.desired.state",
    );
    let feature_descriptor = strategy_descriptor_named(
        CommitStrategyId(42),
        "strategy.aspect.field.reconcile",
        "strategy.aspect",
        "aspect.scalar.field.reconcile",
    );
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(
            CommitStrategyRegistration::new(main_descriptor.clone())
                .expect("main strategy registration"),
        )
        .commit_strategy(
            CommitStrategyRegistration::new(feature_descriptor.clone())
                .expect("feature strategy registration"),
        )
        .build();
    let entity = crate::tests::support::create_entity(request_execution, &runtime, "shared");
    let feature_branch = crate::tests::support::create_branch_from_main(&runtime, "feature");

    {
        let request = runtime
            .commit_strategies()
            .canonicalize_request(
                &IntentReconciliationInput {
                    entity_id: entity,
                    desired_aspect_fields: AspectFieldPatch::from_locator(
                        crate::transactions::data::planned_single_field_locator(
                            worth_foundational::facade::AspectKey::new("name")
                                .expect("valid test aspect key"),
                            FieldKey::new("name").expect("valid test field key"),
                        ),
                        worth_foundational::facade::AspectValue::String("main-strategy".into()),
                    ),
                }
                .into_native_canonical_request(StrategyCallerProvenance {
                    request_origin: StrategyRequestOrigin::Test,
                    actor_identity: None,
                    correlation_id: None,
                })
                .expect("raw main strategy request"),
            )
            .expect("main canonical request");
        let execution = update_execution_draft(&request, entity, "main-strategy");
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime, None);
        let lowered = authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered main strategy plan");
        let validated = authority
            .validate_lowered_plan(request_execution, &runtime, lowered)
            .expect("validated main strategy plan");
        authority
            .execute_validated_commit(request_execution, &runtime, validated)
            .expect("main strategy commit");
    }

    {
        let request = runtime
            .commit_strategies()
            .canonicalize_request(
                &AspectFieldReconciliationInput {
                    entity_id: entity,
                    field_locator: strategy_field_locator(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                    ),
                    desired_value: worth_foundational::facade::AspectValue::String(
                        "feature-strategy".into(),
                    ),
                }
                .into_native_canonical_request(StrategyCallerProvenance {
                    request_origin: StrategyRequestOrigin::Test,
                    actor_identity: None,
                    correlation_id: None,
                })
                .expect("raw feature strategy request"),
            )
            .expect("feature canonical request");
        let execution = update_execution_draft(&request, entity, "feature-strategy");
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(
                &runtime,
                Some(feature_branch.clone()),
            );
        let lowered = authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered feature strategy plan");
        let validated = authority
            .validate_lowered_plan(request_execution, &runtime, lowered)
            .expect("validated feature strategy plan");
        authority
            .execute_validated_commit(request_execution, &runtime, validated)
            .expect("feature strategy commit");
    }

    let planning = runtime
        .merge()
        .inspect_planning_scope(
            request_execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                feature_branch,
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("merge planning artifact");

    let classification = planning
        .conflict_classification
        .classifications
        .iter()
        .find(|classification| {
            classification.record == crate::facade::transactions::RecordRef::Entity(entity)
        })
        .expect("classified shared entity");

    assert_eq!(
        classification.class,
        crate::merge::data::MergeConflictClass::StrategyIntentConflict
    );
    let strategy_evidence = classification
        .strategy_evidence
        .as_ref()
        .expect("strategy conflict evidence");
    assert_eq!(
        strategy_evidence.class,
        crate::merge::data::StrategyConflictClass::DifferentStrategyOverlappingIntent
    );
    assert_eq!(strategy_evidence.source_descriptors.len(), 1);
    assert_eq!(strategy_evidence.target_descriptors.len(), 1);
    assert_eq!(
        planning
            .conflict_classification
            .strategy_intent_conflict_count,
        1
    );
}

#[test]
fn merge_planning_with_real_strategies_preserves_strategy_specific_manual_boundary() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let intent_descriptor = IntentReconciliationStrategy::descriptor(CommitStrategyId(71));
    let replica_descriptor = ReplicaConvergenceStrategy::descriptor(CommitStrategyId(72));
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(
            CommitStrategyRegistration::new(intent_descriptor.clone())
                .expect("intent strategy registration"),
        )
        .commit_strategy_executor(IntentReconciliationStrategy::execution_registration(
            &intent_descriptor,
        ))
        .commit_strategy(
            CommitStrategyRegistration::new(replica_descriptor.clone())
                .expect("replica strategy registration"),
        )
        .commit_strategy_executor(ReplicaConvergenceStrategy::execution_registration(
            &replica_descriptor,
        ))
        .build();
    let entity = crate::tests::support::create_entity(request_execution, &runtime, "shared");
    let feature_branch = crate::tests::support::create_branch_from_main(&runtime, "feature-real");

    {
        let request = runtime
            .commit_strategies()
            .canonicalize_request(
                &IntentReconciliationInput {
                    entity_id: entity,
                    desired_aspect_fields: strategy_name_and_replicas_patch("main-intent", 1),
                }
                .into_native_canonical_request(StrategyCallerProvenance {
                    request_origin: StrategyRequestOrigin::Test,
                    actor_identity: None,
                    correlation_id: None,
                })
                .expect("native canonical strategy request"),
            )
            .expect("intent canonical request");
        let snapshot = runtime.visibility_authority().snapshot();
        let execution = runtime
            .commit_strategies()
            .execute(&request, &snapshot, request_execution)
            .expect("intent execution");
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime, None);
        let lowered = authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered intent plan");
        let validated = authority
            .validate_lowered_plan(request_execution, &runtime, lowered)
            .expect("validated intent plan");
        authority
            .execute_validated_commit(request_execution, &runtime, validated)
            .expect("intent strategy commit");
    }

    {
        let request = runtime
            .commit_strategies()
            .canonicalize_request(
                &ReplicaConvergenceInput {
                    entity_id: entity,
                    desired_replicas: 7,
                }
                .into_native_canonical_request(StrategyCallerProvenance {
                    request_origin: StrategyRequestOrigin::Test,
                    actor_identity: None,
                    correlation_id: None,
                })
                .expect("native canonical strategy request"),
            )
            .expect("replica canonical request");
        let snapshot = runtime.visibility_authority().snapshot();
        let execution = runtime
            .commit_strategies()
            .execute(&request, &snapshot, request_execution)
            .expect("replica execution");
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(
                &runtime,
                Some(feature_branch.clone()),
            );
        let lowered = authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered replica plan");
        let validated = authority
            .validate_lowered_plan(request_execution, &runtime, lowered)
            .expect("validated replica plan");
        authority
            .execute_validated_commit(request_execution, &runtime, validated)
            .expect("replica strategy commit");
    }

    let lowered = runtime
        .merge()
        .inspect_planning_scope(
            request_execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                feature_branch,
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("merge planning scope");

    let classification_index = lowered
        .conflict_classification
        .classifications
        .iter()
        .position(|classification| {
            classification.record == crate::transactions::data::RecordRef::Entity(entity)
        })
        .expect("entity conflict classification index");
    let policy_record = lowered
        .policy_resolution
        .records
        .iter()
        .find(|record| record.record == crate::transactions::data::RecordRef::Entity(entity))
        .expect("entity policy record");

    assert_eq!(
        lowered.conflict_classification.classifications[classification_index].class,
        crate::merge::data::MergeConflictClass::StrategyIntentConflict
    );
    assert_eq!(
        policy_record.proof_boundary.decision_boundary,
        crate::merge::data::MergePolicyDecisionBoundary::RequiresManualResolution {
            class: crate::merge::data::MergeManualResolutionClass::StrategyIntentConflict,
        }
    );
    assert!(lowered.digest_basis.lowered_plan.blocked_reasons[classification_index].is_some());
}
