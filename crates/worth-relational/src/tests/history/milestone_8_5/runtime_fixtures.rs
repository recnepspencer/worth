use super::*;

#[derive(Debug, Clone, Copy)]
struct DeterministicFailureExecutor;

impl CommitStrategyExecutor for DeterministicFailureExecutor {
    fn execute(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _request: &crate::commit_strategies::data::CanonicalStrategyCommitRequest,
        _observation: &StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionResult, StrategyExecutorFailure> {
        Err(StrategyExecutorFailure::new(
            StrategyExecutorFailureClass::DomainRejection,
            "milestone-8.5 hostile deterministic executor rejection",
        ))
    }
}

pub(super) fn persisted_strategy_runtime(
    medium: std::sync::Arc<crate::tests::support::TestDurableMedium>,
) -> RelationalRuntime {
    let intent_descriptor = IntentReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(801),
    );
    let replica_descriptor = ReplicaConvergenceStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(802),
    );
    let aspect_descriptor = AspectFieldReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(803),
    );
    let replacement_descriptor = EntityReplacementReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(804),
    );
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(strategy_schema_registry())
        .commit_strategy(
            crate::facade::commit_strategies::CommitStrategyRegistration::new(
                intent_descriptor.clone(),
            )
            .expect("intent registration"),
        )
        .commit_strategy_executor(IntentReconciliationStrategy::execution_registration(
            &intent_descriptor,
        ))
        .commit_strategy(
            crate::facade::commit_strategies::CommitStrategyRegistration::new(
                replica_descriptor.clone(),
            )
            .expect("replica registration"),
        )
        .commit_strategy_executor(ReplicaConvergenceStrategy::execution_registration(
            &replica_descriptor,
        ))
        .commit_strategy(
            crate::facade::commit_strategies::CommitStrategyRegistration::new(
                aspect_descriptor.clone(),
            )
            .expect("aspect registration"),
        )
        .commit_strategy_executor(AspectFieldReconciliationStrategy::execution_registration(
            &aspect_descriptor,
        ))
        .commit_strategy(
            crate::facade::commit_strategies::CommitStrategyRegistration::new(
                replacement_descriptor.clone(),
            )
            .expect("replacement registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(
                &replacement_descriptor,
            ),
        )
        .build_durable(medium.attachment())
}

pub(super) fn persisted_replacement_strategy_runtime(
    medium: std::sync::Arc<crate::tests::support::TestDurableMedium>,
) -> RelationalRuntime {
    let replacement_descriptor = EntityReplacementReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(804),
    );
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(strategy_schema_registry())
        .commit_strategy(
            crate::facade::commit_strategies::CommitStrategyRegistration::new(
                replacement_descriptor.clone(),
            )
            .expect("replacement registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(
                &replacement_descriptor,
            ),
        )
        .build_durable(medium.attachment())
}

pub(super) fn persisted_strategy_runtime_without_executors(
    medium: std::sync::Arc<crate::tests::support::TestDurableMedium>,
) -> RelationalRuntime {
    let intent_descriptor = IntentReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(801),
    );
    let replica_descriptor = ReplicaConvergenceStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(802),
    );
    let aspect_descriptor = AspectFieldReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(803),
    );
    let replacement_descriptor = EntityReplacementReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(804),
    );
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(strategy_schema_registry())
        .commit_strategy(
            CommitStrategyRegistration::new(intent_descriptor).expect("intent registration"),
        )
        .commit_strategy(
            CommitStrategyRegistration::new(replica_descriptor).expect("replica registration"),
        )
        .commit_strategy(
            CommitStrategyRegistration::new(aspect_descriptor).expect("aspect registration"),
        )
        .commit_strategy(
            CommitStrategyRegistration::new(replacement_descriptor)
                .expect("replacement registration"),
        )
        .build_durable(medium.attachment())
}

pub(super) fn persisted_strategy_runtime_with_failing_intent_executor(
    medium: std::sync::Arc<crate::tests::support::TestDurableMedium>,
) -> RelationalRuntime {
    let intent_descriptor = IntentReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(801),
    );
    let replica_descriptor = ReplicaConvergenceStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(802),
    );
    let aspect_descriptor = AspectFieldReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(803),
    );
    let replacement_descriptor = EntityReplacementReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(804),
    );
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(strategy_schema_registry())
        .commit_strategy(
            CommitStrategyRegistration::new(intent_descriptor.clone())
                .expect("intent registration"),
        )
        .commit_strategy_executor(CommitStrategyExecutionRegistration::new(
            &intent_descriptor,
            DeterministicFailureExecutor,
        ))
        .commit_strategy(
            CommitStrategyRegistration::new(replica_descriptor.clone())
                .expect("replica registration"),
        )
        .commit_strategy_executor(ReplicaConvergenceStrategy::execution_registration(
            &replica_descriptor,
        ))
        .commit_strategy(
            CommitStrategyRegistration::new(aspect_descriptor.clone())
                .expect("aspect registration"),
        )
        .commit_strategy_executor(AspectFieldReconciliationStrategy::execution_registration(
            &aspect_descriptor,
        ))
        .commit_strategy(
            CommitStrategyRegistration::new(replacement_descriptor.clone())
                .expect("replacement registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(
                &replacement_descriptor,
            ),
        )
        .build_durable(medium.attachment())
}

fn strategy_schema_registry() -> crate::schema::data::RelationalSchemaRegistry {
    AspectSchemaFixture {
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
    .build_registry()
}

pub(super) fn execute_strategy_commit(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    request: NativeStrategyCommitRequest,
    target_branch: Option<BranchId>,
) -> crate::facade::transactions::CommitResult {
    let request = runtime
        .commit_strategies()
        .canonicalize_request(&request)
        .expect("canonical strategy request");
    let snapshot = target_branch
        .as_ref()
        .map(|branch| crate::tests::support::snapshot_for_owner_branch(runtime, branch))
        .unwrap_or_else(|| runtime.visibility_authority().snapshot());
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot, request_execution)
        .expect("strategy execution");
    let transaction_validation_input = target_branch
        .as_ref()
        .map(|branch| {
            crate::tests::support::test_owner_transaction_validation_input_for_branch(
                runtime,
                branch.clone(),
            )
        })
        .unwrap_or_else(|| {
            crate::tests::support::test_owner_transaction_validation_input_for_main(runtime)
        });
    let mut authority = runtime.commit_strategies_authority();
    let lowered = authority
        .lower_execution_with_input(runtime, &request, &execution, transaction_validation_input)
        .expect("lowered strategy plan");
    let validated = authority
        .validate_lowered_plan(request_execution, runtime, lowered)
        .expect("validated strategy plan");
    authority
        .execute_validated_commit(request_execution, runtime, validated)
        .expect("strategy commit")
}
