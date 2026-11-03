use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::commit_refusals::refused_commit;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::authority;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::isolated;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::request;
use worth_relational::facade::identity::PartitionId;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphInstallationDenialKind;
use worth_query_execution::domain_computation::primary_graph::bootstrap_publication::commit_denial::map_bootstrap_commit_denial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as QueryKind;
use std::num::NonZeroUsize;
use std::time::Instant;
use worth_execution::CancellationSource;
use worth_execution::LeaseRequest;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
#[test]
fn bootstrap_commit_preserves_real_leased_work_refusal() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    {
        isolated(
            concat!(
                module_path!(),
                "::bootstrap_commit_preserves_real_leased_work_refusal"
            ),
            || {
                let mut request = request();
                request.policy = ExecutionRequestPolicy::new(
                    ExecutionPosture::Automatic,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, None, Some(0)),
                );
                let error = refused_commit(execution, request);
                let worth_relational::facade::transactions::TransactionCommitError::Execution {
                    denial,
                    ..
                } = &error
                else {
                    panic!("real execution refusal required");
                };
                assert_eq!(
                    denial.kind,
                    worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(
                        Cause::WorkExhausted
                    )
                );
                let partition_identity = denial.partition_identity;
                assert_eq!(
                    partition_identity,
                    Some(1),
                    "residency admission attributes the refused Head read to main"
                );
                let validation = worth_query_execution::domain_computation::primary_graph::provider::invariant_execution_failure::map_validation_failure(error.clone());
                assert_eq!(
                validation.kind(),
                worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionDenialKind::ExecutionDenied(
                    QueryKind::ExecutionResource {
                        denial: Resource::WorkExhausted,
                        partition_identity,
                        policy_ancestor: None
                    }
                )
            );
                let denied = map_bootstrap_commit_denial(error);
                assert_eq!(
                    denied.kind(),
                    WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                        kind: QueryKind::ExecutionResource {
                            denial: Resource::WorkExhausted,
                            partition_identity,
                            policy_ancestor: None
                        },
                    }
                );
                let WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied { kind } =
                    denied.kind()
                else {
                    panic!("installation lost the execution refusal");
                };
                let application = worth_query_execution::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_denied(
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure::new(
                kind, worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
                "commit preparation execution refused", Default::default(),
            ),
        );
                let observed = WorthQueryApplicationCommitOutcome::Denied(application)
                    .require_committed()
                    .expect_err("the receipt requirement preserves a refusal");
                let WorthQueryApplicationCommitOutcome::Denied(observed) = observed else {
                    panic!("the receipt requirement folded the refusal");
                };
                assert_eq!(
                observed.kind(),
                worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind::ExecutionResource {
                    denial: Resource::WorkExhausted,
                    partition_identity,
                    policy_ancestor: None,
                }
            );
            },
        );
    }
}
#[test]
fn bootstrap_commit_preserves_real_control_stops() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    isolated(
        concat!(
            module_path!(),
            "::bootstrap_commit_preserves_real_control_stops"
        ),
        || {
            let cancellation = CancellationSource::new();
            cancellation.cancel();
            let mut canceled = request();
            canceled.cancellation = cancellation.token();
            let mut expired = request();
            expired.deadline = Some(Instant::now());
            for (request, expected) in
                [(canceled, Control::Cancelled), (expired, Control::TimedOut)]
            {
                let observed = map_bootstrap_commit_denial(refused_commit(execution, request));
                assert_eq!(
                    observed.kind(),
                    WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped {
                        kind: expected
                    }
                );
            }
        },
    );
}
#[test]
fn bootstrap_operation_interruption_keeps_heads_generic_commit_rejection() {
    use worth_relational::facade::mvcc::RelationalCancellationSource;
    use worth_relational::facade::mvcc::RelationalInterruptionBoundary;
    use worth_relational::facade::mvcc::RelationalOperationControl;
    use worth_relational::facade::transactions::TransactionCommitError;
    let source = RelationalCancellationSource::new();
    source.cancel();
    let canceled: RelationalOperationControl = source.token().into();
    let expired = RelationalOperationControl::uninterrupted().with_deadline(Instant::now());
    for control in [canceled, expired] {
        let event = control
            .observe(RelationalInterruptionBoundary::CandidatePreparation)
            .unwrap();
        let observed = map_bootstrap_commit_denial(TransactionCommitError::interrupted(event));
        assert_eq!(
            observed.kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected
        );
    }
}
