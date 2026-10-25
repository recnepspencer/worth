use super::native_strategy_fixtures::*;

#[test]
fn merge_planning_distinguishes_disjoint_aspect_intent_from_strategy_intent_conflict() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let aspect_descriptor = AspectFieldReconciliationStrategy::descriptor(CommitStrategyId(91));
    let replica_descriptor = ReplicaConvergenceStrategy::descriptor(CommitStrategyId(92));
    let registry = AspectSchemaFixture {
        cascade_delete_policy: crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations,
        entity_aspects: vec![
            entity_field_aspect(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
            ),
            entity_u64_field_aspect(
                crate::tests::support::aspect_key("replicas"),
                crate::tests::support::field_key("replicas"),
            ),
            lifecycle_aspect(),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_registry();
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(registry)
        .commit_strategy(
            CommitStrategyRegistration::new(aspect_descriptor.clone())
                .expect("aspect strategy registration"),
        )
        .commit_strategy_executor(AspectFieldReconciliationStrategy::execution_registration(
            &aspect_descriptor,
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
    let feature_branch =
        crate::tests::support::create_branch_from_main(&runtime, "feature-aspects");

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
                        "main-name".into(),
                    ),
                }
                .into_native_canonical_request(StrategyCallerProvenance {
                    request_origin: StrategyRequestOrigin::Test,
                    actor_identity: None,
                    correlation_id: None,
                })
                .expect("native canonical strategy request"),
            )
            .expect("aspect canonical request");
        let snapshot = runtime.visibility_authority().snapshot();
        let execution = runtime
            .commit_strategies()
            .execute(&request, &snapshot, request_execution)
            .expect("aspect execution");
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime, None);
        let lowered = authority
            .lower_execution_with_input(
                request_execution,
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered aspect plan");
        let validated = authority
            .validate_lowered_plan(request_execution, &runtime, lowered)
            .expect("validated aspect plan");
        authority
            .execute_validated_commit(request_execution, &runtime, validated)
            .expect("aspect strategy commit");
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
                request_execution,
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
            classification.record == crate::transactions::data::RecordRef::Entity(entity)
        })
        .expect("classified shared entity");

    assert_ne!(
        classification.class,
        crate::merge::data::MergeConflictClass::StrategyIntentConflict
    );
    assert!(
        classification.strategy_evidence.is_none(),
        "disjoint declared aspect intent should not synthesize strategy conflict evidence: {classification:?}"
    );
}

#[test]
fn merge_planning_classifies_same_declared_aspect_field_as_strategy_intent_conflict() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let aspect_descriptor = AspectFieldReconciliationStrategy::descriptor(CommitStrategyId(93));
    let registry = AspectSchemaFixture {
        cascade_delete_policy: crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations,
        entity_aspects: vec![
            entity_field_aspect(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
            ),
            entity_u64_field_aspect(
                crate::tests::support::aspect_key("replicas"),
                crate::tests::support::field_key("replicas"),
            ),
            lifecycle_aspect(),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_registry();
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(registry)
        .commit_strategy(
            CommitStrategyRegistration::new(aspect_descriptor.clone())
                .expect("aspect strategy registration"),
        )
        .commit_strategy_executor(AspectFieldReconciliationStrategy::execution_registration(
            &aspect_descriptor,
        ))
        .build();
    let entity = crate::tests::support::create_entity(request_execution, &runtime, "shared");
    let feature_branch =
        crate::tests::support::create_branch_from_main(&runtime, "feature-same-aspect");

    for (branch, desired_value) in [
        (None, "main-name"),
        (Some(feature_branch.clone()), "feature-name"),
    ] {
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
                        desired_value.into(),
                    ),
                }
                .into_native_canonical_request(StrategyCallerProvenance {
                    request_origin: StrategyRequestOrigin::Test,
                    actor_identity: None,
                    correlation_id: None,
                })
                .expect("native canonical strategy request"),
            )
            .expect("aspect canonical request");
        let snapshot = runtime.visibility_authority().snapshot();
        let execution = runtime
            .commit_strategies()
            .execute(&request, &snapshot, request_execution)
            .expect("aspect execution");
        let transaction_validation_input = branch
            .as_ref()
            .map(|branch| {
                crate::tests::support::test_owner_transaction_validation_input_for_branch(
                    &runtime,
                    branch.clone(),
                )
            })
            .unwrap_or_else(|| {
                crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
            });
        let mut authority = runtime.commit_strategies_authority();
        let lowered = authority
            .lower_execution_with_input(
                request_execution,
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered aspect plan");
        let validated = authority
            .validate_lowered_plan(request_execution, &runtime, lowered)
            .expect("validated aspect plan");
        authority
            .execute_validated_commit(request_execution, &runtime, validated)
            .expect("aspect strategy commit");
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
            classification.record == crate::transactions::data::RecordRef::Entity(entity)
        })
        .expect("classified shared entity");

    assert_eq!(
        classification.class,
        crate::merge::data::MergeConflictClass::StrategyIntentConflict
    );
    assert_eq!(
        classification
            .strategy_evidence
            .as_ref()
            .expect("strategy conflict evidence")
            .class,
        crate::merge::data::StrategyConflictClass::SameStrategyDivergentOutput
    );
}
