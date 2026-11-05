// A request stopped before registration admits its decision-fact binding
// keeps that control outcome and releases every attempt-local resource.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::WorthQueryProgressedApplicationCommit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::session_admission::admit_provider_session;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::session_admission::registration_conflict_tests::equivalent_programs;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::session_admission::registration_conflict_tests::start;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progressed::finish_application_commit;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryProviderAttemptRegistrationContext;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[test]
fn cancellation_at_registration_binding_is_cancelled_not_denied() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world
        .application
        .with_host_advancement(|active_phase| {
            let host_custody = active_phase.test_host_request_for_test();
            let phase = &active_phase;
            let head = || {
                world
                    .selected_product(&host_custody)
                    .product()
                    .selected_commit()
                    .clone()
            };
            let head_before = head();
            let (program, _) = equivalent_programs(
                &host_custody.phase_for_application_for_test(&world.application),
                &world,
                &request,
                "interrupted-registration",
            );
            let (admission, lease, provider_attempt, outcome_identity, _, mut authorization, idempotency, mut running, mutation_run, mut attempt_basis, aftermath_causality, completion_storage,) = start(phase, &world, program, idempotency(185, 186)).into_parts_for_test();
    let original_completion = attempt_basis.take_original_completion_for_test();
            let admitted = admit_provider_session(
                phase,
                &mut running,
                (*world.application.runtime_for_test()).primary_graph_authority_for_test(),
                attempt_basis.retained_product(),
                mutation_run,
            )
            .unwrap_or_else(|_| panic!("the live session must reach provider registration"));
            // The request stops after session admission, so the first owner
            // consult to observe it is registration's decision-fact binding.
            cancellation.cancel();
            let Err(failure) = admitted.register_for_test(
                &mut authorization,
                provider_attempt,
                attempt_basis,
                WorthQueryProviderAttemptRegistrationContext::new_for_test(
                    &(*(*world.application.runtime_for_test()).primary_provider_for_test()),
                    &admission,
                    idempotency,
                    outcome_identity,
                    aftermath_causality.as_ref(),
                ),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            ) else {
                panic!("a cancelled request must not register its provider attempt")
            };
            let (outcome, cleanup,) =
                failure.into_completion_for_test().into_parts_for_test();
            assert!(
                matches!(outcome, WorthQueryProviderProgressionOutcome::Cancelled),
                "the binding's request stop is a control outcome, never a denial"
            );
            let finished = finish_application_commit(
                &phase.borrowed_request_for_test(),
                phase,
                &world.application,
        completion_storage.complete_for_test(WorthQueryProgressedApplicationCommit::new_for_test(None, outcome, lease, running, cleanup, admission.publication_request_for_test().clone(), original_completion)),
    );
            assert!(matches!(
                finished,
                WorthQueryApplicationCommitOutcome::Cancelled
            ));
            assert_eq!(
                (*(*world
                    .application
                    .runtime_for_test())
                    .primary_provider_for_test())
                    .application_attempt_resource_count_for_test(),
                0
            );
            assert_eq!(
                world.application.provider_session_resource_count_for_test(),
                0
            );
            assert_eq!(head(), head_before, "nothing may commit");
        })
        .expect("fixture owner admits its advancement");
}
