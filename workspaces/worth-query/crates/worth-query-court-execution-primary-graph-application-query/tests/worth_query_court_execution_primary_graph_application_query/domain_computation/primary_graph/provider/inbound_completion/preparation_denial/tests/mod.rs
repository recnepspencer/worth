use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::inbound_completion::WorthQueryInboundCompletionPreparationDenial as Denial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage as Stage;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::inbound_completion::preparation_denial::preparation_denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::inbound_completion::preparation_denial::validation_denial;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::commit_refusals::refused_commit;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::commit_refusals::work_refusal;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::isolated;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::request;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
use worth_execution::CancellationSource;

#[test]
fn inbound_completion_preserves_real_execution_refusals_at_each_phase() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    isolated(
        concat!(
            module_path!(),
            "::inbound_completion_preserves_real_execution_refusals_at_each_phase"
        ),
        || {
            let error = work_refusal(execution);
            let expected = Kind::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: None,
                policy_ancestor: None,
            };
            assert_eq!(
                validation_denial(&error),
                Denial::ExecutionDenied {
                    stage: Stage::InvariantExecution,
                    kind: expected
                }
            );
            assert_eq!(
                preparation_denial(&error),
                Denial::ExecutionDenied {
                    stage: Stage::ProviderCommit,
                    kind: expected
                }
            );
            let source = CancellationSource::new();
            source.cancel();
            let mut canceled = request();
            canceled.cancellation = source.token();
            let mut expired = request();
            expired.deadline = Some({
                let clock = worth_execution::MonotonicClock::system();
                clock.deadline_at(clock.now())
            });
            for (request, kind) in [(canceled, Control::Cancelled), (expired, Control::TimedOut)] {
                let error = refused_commit(execution, request);
                assert_eq!(
                    validation_denial(&error),
                    Denial::ExecutionControlStopped {
                        stage: Stage::InvariantExecution,
                        kind
                    }
                );
                assert_eq!(
                    preparation_denial(&error),
                    Denial::ExecutionControlStopped {
                        stage: Stage::ProviderCommit,
                        kind
                    }
                );
            }
        },
    );
}
