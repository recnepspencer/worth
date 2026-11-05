//! Real owner refusals cross both production completion consumer seams.
use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::installed_world;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect;
use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchRequest;
use worth_query_execution::facade::domain_computation::WorthQueryExternalEffectTransport;
use worth_query_execution::facade::domain_computation::WorthQueryExternalTransportOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::custody::InstalledTransportPendingReason as Pending;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::progression::completion_candidate as authenticated_completion_candidate;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::transport_progression::completion_candidate as transport_completion_candidate;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::installed_transport::InstalledTransportPublicationOutcome as TransportOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::outcome::WorthQueryInboundPublicationDenial as PublicationDenial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::outcome::WorthQueryInboundPublicationOutcome as PublicationOutcome;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::commit_refusals::refused_commit;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::commit_refusals::work_refusal;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::isolated;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::request;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::inbound_completion::preparation_denial::preparation_denial as completion_preparation_denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::inbound_completion::preparation_denial::validation_denial as completion_validation_denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::inbound_completion::WorthQueryInboundCompletionPreparationDenial as Preparation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::installed_transport::InstalledTransportCompletion;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage as Stage;
use worth_query_execution::facade::domain_computation::WorthQueryInboundAdmissionDenial as Retry;
use worth_query_execution::facade::domain_computation::WorthQueryInboundPendingReason as ReceiptPending;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
use std::sync::Arc;
use worth_execution::CancellationSource;

pub struct CompletingTransport;
impl WorthQueryExternalEffectTransport for CompletingTransport {
    fn dispatch(
        &self,
        _: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        WorthQueryExternalTransportOutcome::Completed
    }
}

#[test]
fn inbound_publication_consumers_keep_real_execution_refusals_and_retry_posture() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    {
        isolated(
            concat!(
                module_path!(),
                "::inbound_publication_consumers_keep_real_execution_refusals_and_retry_posture"
            ),
            || {
                // No lease is threaded into production here. The candidate-result seam
                // receives a real Relational refusal; transport evidence is owner-issued.
                let world = installed_world();
                let receipt = world.commit_dispatch(71, "completion-execution-refusal");
                let owner = world
                    .application
                    .with_host_advancement(|phase| {
                        let host_custody = phase.test_host_request_for_test();

                        world
                            .application
                            .observe_committed_dispatch_outbox(
                                &host_custody.phase_for_application_for_test(&world.application),
                                &receipt,
                            )
                            .unwrap()
                            .unwrap()
                    })
                    .expect("the fixture read owns its installed host call");
                let admitted = world
                    .application
                    .admit_external_dispatch_attempt_for_test(owner.clone())
                    .unwrap();
                let completed =
                    dispatch_external_effect(&CompletingTransport, admitted, || {}).unwrap();
                let evidence = Arc::new(
                    InstalledTransportCompletion::from_observed_dispatch_for_test(
                        owner, &completed,
                    )
                    .unwrap(),
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
                let refusals = [
                    (
                        work_refusal(execution),
                        Ok(Kind::ExecutionResource {
                            denial: Resource::WorkExhausted,
                            partition_identity: None,
                            policy_ancestor: None,
                        }),
                    ),
                    (refused_commit(execution, canceled), Err(Control::Cancelled)),
                    (refused_commit(execution, expired), Err(Control::TimedOut)),
                ];
                for (error, expected) in refusals {
                    for (preparation, stage) in [
                        (completion_preparation_denial(&error), Stage::ProviderCommit),
                        (
                            completion_validation_denial(&error),
                            Stage::InvariantExecution,
                        ),
                    ] {
                        let expected_preparation = match expected {
                            Ok(kind) => Preparation::ExecutionDenied { stage, kind },
                            Err(kind) => Preparation::ExecutionControlStopped { stage, kind },
                        };
                        assert_eq!(preparation, expected_preparation);
                        let Err(PublicationOutcome::Denied(
                            PublicationDenial::CompletionPreparation(observed),
                        )) = authenticated_completion_candidate(Err(preparation))
                        else {
                            panic!(
                                "authenticated publication consumer discarded the owner refusal"
                            );
                        };
                        assert_eq!(observed, expected_preparation);
                        let observed = PublicationDenial::CompletionPreparation(observed)
                            .retry_denial_for_test();
                        match (expected, observed.pending_reason_for_test()) {
                            (
                                Ok(kind),
                                ReceiptPending::ExecutionDenied {
                                    stage: actual,
                                    kind: cause,
                                },
                            ) => {
                                assert_eq!(actual, stage);
                                assert_eq!(cause, kind);
                            }
                            (
                                Err(kind),
                                ReceiptPending::ExecutionControlStopped {
                                    stage: actual,
                                    kind: cause,
                                },
                            ) => {
                                assert_eq!(actual, stage);
                                assert_eq!(cause, kind);
                            }
                            _ => panic!("accepted receipt folded the cause"),
                        }
                        match (expected, observed) {
                            (
                                Ok(kind),
                                Retry::PublicationExecutionDenied {
                                    stage: actual,
                                    kind: cause,
                                },
                            ) => {
                                assert_eq!(actual, stage);
                                assert_eq!(cause, kind);
                            }
                            (
                                Err(kind),
                                Retry::PublicationExecutionControlStopped {
                                    stage: actual,
                                    kind: cause,
                                },
                            ) => {
                                assert_eq!(actual, stage);
                                assert_eq!(cause, kind);
                            }
                            _ => panic!("authenticated retry consumer folded the cause"),
                        }
                        let Err(TransportOutcome::Denied(retained, denial)) =
                            transport_completion_candidate(Arc::clone(&evidence), Err(preparation))
                        else {
                            panic!("transport publication consumer discarded the owner refusal");
                        };
                        assert!(
                            Arc::ptr_eq(&retained, &evidence),
                            "real dispatch evidence changed"
                        );
                        let pending = denial.pending_reason_for_test();
                        let recovery = worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandleDenial::from(
                        pending.dispatch_preparation_denial_for_test().redispatch_denial_for_test(),
                    ).kind();
                        use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandleDenialKind as Recovery;
                        match (expected, recovery) {
                            (
                                Ok(kind),
                                Recovery::CompletionExecutionDenied {
                                    stage: actual,
                                    kind: cause,
                                },
                            ) => {
                                assert_eq!((actual, cause), (stage, kind));
                            }
                            (
                                Err(kind),
                                Recovery::CompletionExecutionControlStopped {
                                    stage: actual,
                                    kind: cause,
                                },
                            ) => {
                                assert_eq!((actual, cause), (stage, kind));
                            }
                            _ => panic!("redispatch recovery consumer folded the cause"),
                        }
                        match (expected, pending) {
                            (
                                Ok(kind),
                                Pending::ExecutionDenied {
                                    stage: actual,
                                    kind: cause,
                                },
                            ) => {
                                assert_eq!(actual, stage);
                                assert_eq!(cause, kind);
                            }
                            (
                                Err(kind),
                                Pending::ExecutionControlStopped {
                                    stage: actual,
                                    kind: cause,
                                },
                            ) => {
                                assert_eq!(actual, stage);
                                assert_eq!(cause, kind);
                            }
                            _ => panic!("transport retry consumer folded the cause"),
                        }
                    }
                }
            },
        );
    }
}
