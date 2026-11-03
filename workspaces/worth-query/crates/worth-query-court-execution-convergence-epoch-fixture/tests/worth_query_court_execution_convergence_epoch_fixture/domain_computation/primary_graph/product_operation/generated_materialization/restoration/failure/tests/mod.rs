use worth_relational::facade::branch::RelationalMaterializationError;
use worth_query_execution::facade::domain_computation::WorthQueryGeneratedOutputRestorationFailureCause;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::generated_materialization::restoration::failure::preparation_failure_cause;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::commit_refusals::refused_commit;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::commit_refusals::work_refusal;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::isolated;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::request;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
use std::time::Instant;
use worth_execution::CancellationSource;

#[test]
fn restoration_preserves_real_execution_refusals() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    {
        isolated(
            concat!(
                module_path!(),
                "::restoration_preserves_real_execution_refusals"
            ),
            || {
                let error = RelationalMaterializationError::Commit(work_refusal(execution));
                let WorthQueryGeneratedOutputRestorationFailureCause::ExecutionDenied(kind) =
                    preparation_failure_cause(&error)
                else {
                    panic!("restoration folded a real execution refusal");
                };
                assert_eq!(
                    kind,
                    Kind::ExecutionResource {
                        denial: Resource::WorkExhausted,
                        partition_identity: Some(1),
                        policy_ancestor: None
                    }
                );
                let suspension =
                    worth_query_execution::facade::domain_computation::WorthQueryGeneratedOutputSuspensionFailure::Preparation(
                        error,
                    );
                assert_eq!(
                    suspension.execution_control_stop(),
                    None,
                    "a resource refusal is not a control stop"
                );
                let source = CancellationSource::new();
                source.cancel();
                let mut canceled = request();
                canceled.cancellation = source.token();
                let mut expired = request();
                expired.deadline = Some(Instant::now());
                for (request, expected) in
                    [(canceled, Control::Cancelled), (expired, Control::TimedOut)]
                {
                    let error =
                        RelationalMaterializationError::Commit(refused_commit(execution, request));
                    let WorthQueryGeneratedOutputRestorationFailureCause::ExecutionControlStopped(
                        kind,
                    ) = preparation_failure_cause(&error)
                    else {
                        panic!("restoration folded control stop");
                    };
                    assert_eq!(kind, expected);
                    let suspension = worth_query_execution::facade::domain_computation::WorthQueryGeneratedOutputSuspensionFailure::Preparation(error);
                    assert_eq!(suspension.execution_control_stop(), Some(expected));
                    let worth_query_execution::facade::domain_computation::WorthQueryGeneratedOutputSuspensionFailure::Preparation(retained) = suspension else { panic!("borrowed stop projection must retain original Native error") };
                    assert!(
                        matches!(preparation_failure_cause(&retained), WorthQueryGeneratedOutputRestorationFailureCause::ExecutionControlStopped(kind) if kind == expected)
                    );
                }
            },
        );
    }
}
