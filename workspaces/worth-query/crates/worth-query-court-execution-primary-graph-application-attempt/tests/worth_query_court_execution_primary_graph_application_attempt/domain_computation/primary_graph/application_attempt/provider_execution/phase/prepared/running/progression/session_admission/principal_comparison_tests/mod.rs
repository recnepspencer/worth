use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::comparison_tests::ComparisonTestsWorthQueryRegisteredProviderAttemptForTest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::session_admission::admit_provider_session;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryProviderAttemptRegistrationContext;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::commit_preparation::prepare_application_commit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::start_managed_application_commit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparationRequest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::WorthQueryRunningApplicationCommit;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;

#[test]
fn principal_staleness_and_request_stop_keep_their_distinct_answers() {
    let world = installed_authorization_world(true);
    let cancellation =
        worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new();
    let request =
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
            Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
            cancellation.token(),
        );
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            retained_status_program(
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &account,
                &request,
                "overlay-owner",
                RetentionMutationBreadth::Narrow,
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_application_advancement(
            &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope(),
            |active_phase| {
                let phase = &active_phase;
                let prepared = prepare_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    WorthQueryApplicationCommitPreparationRequest::new_for_test(
                        program,
                        idempotency(187, 188),
                        None,
                        None,
                    ),
                );
                let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepared else {
                    panic!("overlay fixture must reach ordinary prepared posture")
                };
                let running = start_managed_application_commit(phase, &world.application, prepared)
                    .unwrap_or_else(|outcome| panic!("overlay fixture must start: {outcome:?}"));
                let (admission, lease, provider_attempt, outcome_identity, _workflow_settlement_publication, mut authorization, retained_idempotency, mut running, mutation_run, attempt_basis, aftermath_causality,) = running.into_parts_for_test();
                let admitted_session = admit_provider_session(
                    phase,
                    &mut running,
                    &(*world.application.primary_graph_authority_for_test()),
                    attempt_basis.retained_product(),
                    mutation_run,
                )
                .unwrap_or_else(|_| panic!("overlay fixture must admit its real provider session"));
                let (registered, mutation_run,) = admitted_session
                    .register_for_test(
                        &mut authorization,
                        provider_attempt,
                        attempt_basis,
                        WorthQueryProviderAttemptRegistrationContext::new_for_test(
                            &(*world.application.primary_provider_for_test()),
                            &admission,
                            retained_idempotency,
                            outcome_identity,
                            aftermath_causality.as_ref(),
                        ),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap_or_else(|_| panic!("overlay fixture must register")).into_parts_for_test();
                registered.assert_principal_comparison(
                    &phase.borrowed_request_for_test(),
                    &world,
                    &request,
                    principal.target_relation_id_for_test(),
                );
                mutation_run
                    .finish_for_test(
                        running,
                        worth_query_execution::facade::domain_computation::WorthQueryManagedRunTerminalKind::Failed,
                        lease.release_custody(),
                        false,
                    )
                    .unwrap();
            },
        )
        .unwrap();
}
