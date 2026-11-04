use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::session_admission::admit_provider_session;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryProviderAttemptRegistrationContext;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::preparation::prepare_application_commit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::start_managed_application_commit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::WorthQueryApplicationCommitPreparationRequest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::WorthQueryRunningApplicationCommit;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;

#[test]
fn second_real_overlay_is_rejected_without_orphaning_the_first_overlay() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    world
        .application
        .with_host_advancement(|active_phase| {
            let host_custody = active_phase.test_host_request_for_test();
            let phase = &active_phase;
            let execution = phase;

            let request = live_scope();
            let principal = authenticated_principal(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
                &world,
                &request,
            );
            let account = resolved_account(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
                &world,
                "open",
                &request,
            );
            let program = retained_status_program(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
                &world,
                &principal,
                &account,
                &request,
                "overlay-owner",
                RetentionMutationBreadth::Narrow,
            );
            let prepared = prepare_application_commit(
                &phase.borrowed_request_for_test(),
                WorthQueryApplicationCommitPreparationRequest::new_for_test(
                    world
                        .application
                        .validate_commit_program_in_advancement_for_test(phase, program)
                        .unwrap_or_else(|outcome| panic!("fixture admission: {outcome:?}")),
                    idempotency(185, 186),
                    None,
                    None,
                ),
            );
            let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepared else {
                panic!("overlay fixture must reach ordinary prepared posture")
            };
            let running = start_managed_application_commit(phase, &world.application, prepared)
                .unwrap_or_else(|outcome| panic!("overlay fixture must start: {outcome:?}"));
            let (admission, _lease, provider_attempt, outcome_identity, _workflow_settlement_publication, mut authorization, idempotency, mut running, mutation_run, attempt_basis, aftermath_causality,) = running.into_parts_for_test();
            let admitted_session = admit_provider_session(
                execution,
                &mut running,
                (*world.application.runtime_for_test()).primary_graph_authority_for_test(),
                attempt_basis.retained_product(),
                mutation_run,
            )
            .unwrap_or_else(|_| panic!("overlay fixture must admit its real provider session"));
            let registered_session = admitted_session
                .register_for_test(
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
                )
                .unwrap_or_else(|_| panic!("overlay fixture must register"));
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::overlay_conflict_tests::assert_second_real_overlay_is_rejected(
                &phase.borrowed_request_for_test(),
                registered_session.registered_into_for_test(),
                &world,
            );
        })
        .expect("fixture owner admits its advancement");
}
