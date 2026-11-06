use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::comparison_tests::ComparisonTestsWorthQueryRegisteredProviderAttemptForTest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::session_admission::admit_provider_session;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryProviderAttemptRegistrationContext;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::preparation::prepare_application_commit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::start_managed_application_commit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparationRequest;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;

#[test]
fn primary_comparison_scope_preserves_registration_cancellation_cleanup_and_retry() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let foreign = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let cancellation =
        worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new();
    let request =
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
            Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
            cancellation.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
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
    // The caller lease funds the whole attempt, so its capture grants are
    // charged inside the attempt's own worker lineage.
    let caller = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(64 << 20),
                None,
            ),
        ),
        cancellation: worth_execution::CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    world
        .application
        .with_funded_application_advancement(
            &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope(),
            worth_execution::ExecutionAllocationPolicy::Execution(&caller),
            |active_phase| {
                let phase = &active_phase;
                let prepared = prepare_application_commit(
                    &phase.borrowed_request_for_test(),
                    WorthQueryApplicationCommitPreparationRequest::new_for_test(
                        world
                            .application
                            .validate_commit_program_in_advancement_for_test(phase, program)
                            .unwrap_or_else(|outcome| panic!("fixture admission: {outcome:?}")),
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
                let (_coupled_outputs, admission, lease, provider_attempt, outcome_identity, _workflow_settlement_publication, mut authorization, retained_idempotency, mut running, mutation_run, attempt_basis, aftermath_causality,) = running.into_parts_for_test();
                let admitted_session = admit_provider_session(
                    phase,
                    &mut running,
                    (*world.application.runtime_for_test()).primary_graph_authority_for_test(),
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
                            (*world.application.runtime_for_test()).primary_provider_for_test(),
                            &admission,
                            retained_idempotency,
                            outcome_identity,
                            aftermath_causality.as_ref(),
                        ),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap_or_else(|_| panic!("overlay fixture must register")).into_parts_for_test();
                registered.assert_scoped_comparison(
                    &phase.borrowed_request_for_test(),
                    &world,
                    &foreign,
                    &request,
                    &cancellation,
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
    // The retry is a separate host call after the interrupted attempt closed.
    let fresh = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &fresh)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &fresh,
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            retained_status_program(
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &account,
                &fresh,
                "after-retry",
                RetentionMutationBreadth::Narrow,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(
        matches!(
            world.application.compare_and_commit_application(
                retry,
                idempotency(187, 188),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation
            ),
            worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome::Committed(
                _
            )
        ),
        "interrupted comparison must leave the key reusable"
    );
}
