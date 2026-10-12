use super::*;

#[test]
fn projection_contract_violation_is_typed_not_panic_bucket() {
    let host_request = crate::test_execution_policy::serial_request();
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
            observation: &crate::commit_strategies::data::StrategyObservationContext<'_>,
        ) -> Result<StrategyExecutionResult, crate::commit_strategies::data::StrategyExecutorFailure>
        {
            let _ = observation.visibility().entity::<MissingAspectProjection>(
                _execution,
                EntityId::new(PartitionId(1), 0, 0),
            )?;
            Ok(empty_result())
        }
    }

    let executor_binding =
        CommitStrategyExecutionRegistration::new(&descriptor, ProjectionViolationExecutor);
    let runtime = RelationalRuntimeBuilder::new()
        .commit_strategy(registration)
        .commit_strategy_executor(executor_binding)
        .build();
    let snapshot = runtime.visibility_authority().snapshot();
    let request = canonical_request();

    let bound = bind_execution(&runtime, &request, &snapshot, execution).expect("bind strategy");
    let error = execute_bound_strategy(execution, bound).unwrap_err();
    assert!(matches!(
        error,
        StrategyExecutionError::ExecutorFailed {
            strategy_id: CommitStrategyId(41),
            failure,
        } if matches!(&failure, crate::commit_strategies::data::StrategyExecutorFailure::Domain { class, .. } if *class
            == crate::commit_strategies::data::StrategyExecutorFailureClass::ProjectionContractViolation)
    ));
}
