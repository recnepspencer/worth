use super::*;

#[test]
fn projection_contract_violation_is_typed_not_panic_bucket() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = descriptor();
    let registration = CommitStrategyRegistration::new(descriptor.clone()).unwrap();

    #[derive(Clone, Copy)]
    struct ProjectionViolationExecutor;

    impl CommitStrategyExecutor for ProjectionViolationExecutor {
        fn execute(
            &self,
            _execution: &worth_execution::ExecutionRequest<'_, '_>,
            _request: &CanonicalStrategyCommitRequest,
            observation: &worth_relational::facade::commit_strategies::StrategyObservationContext<
                '_,
            >,
        ) -> Result<
            StrategyExecutionResult,
            worth_relational::facade::commit_strategies::StrategyExecutorFailure,
        > {
            let _ = observation.visibility().entity::<MissingAspectProjection>(
                _execution,
                EntityId::new(PartitionId(1), 0, 0),
            )?;
            Ok(empty_result())
        }
    }

    let executor_binding =
        CommitStrategyExecutionRegistration::new(&descriptor, ProjectionViolationExecutor);
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .commit_strategy(registration)
        .commit_strategy_executor(executor_binding)
        .build()
        .expect("valid native runtime configuration");
    let snapshot = current_snapshot(&runtime);
    let request = canonical_request_for(&runtime, &descriptor);

    let error = runtime
        .commit_strategies()
        .execute(&request, &snapshot, execution)
        .unwrap_err();
    let StrategyExecutionError::ExecutorFailed {
        strategy_id: CommitStrategyId(41),
        failure: worth_relational::facade::commit_strategies::StrategyExecutorFailure::Domain {
            class: worth_relational::facade::commit_strategies::StrategyExecutorFailureClass::ProjectionContractViolation,
            evidence: Some(evidence), ..
        },
    } = error else {
        panic!("expected the typed projection contract refusal, got {error:?}");
    };
    let evidence = serde_json::to_value(evidence).unwrap();
    let denial: worth_relational::facade::runtime::RelationalProjectionContractDenial =
        serde_json::from_value(evidence["ProjectionContract"]["denial"].clone()).unwrap();
    assert_eq!(
        denial.domain,
        worth_relational::facade::runtime::ProjectionRecordDomain::Entity
    );
    assert_eq!(denial.kind_id, KindId(100));
    assert!(denial.requirement.is_none());
    assert_eq!(
        denial.refusal,
        worth_relational::facade::runtime::ProjectionContractRefusal::MissingAspectPlan
    );
}
