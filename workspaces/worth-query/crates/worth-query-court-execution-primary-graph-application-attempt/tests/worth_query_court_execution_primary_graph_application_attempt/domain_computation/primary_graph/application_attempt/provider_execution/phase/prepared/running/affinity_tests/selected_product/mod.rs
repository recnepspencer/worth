//! Provider-plan affinity to the exact installed product.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::context::court_doors_for_test::ContextDoorsOps;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::affinity_tests::*;
#[test]
fn an_exact_sibling_product_cannot_substitute_at_provider_plan_binding() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    world
        .application
        .with_host_advancement(|active_phase| {
            let host_custody = active_phase.test_host_request_for_test();
            let phase = &active_phase;

            let running = start(
                phase,
                &world,
                retained_program(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    "product-owner",
                ),
                idempotency(193, 194),
            );
            let (
                _admission,
                lease,
                _provider_attempt,
                _outcome_identity,
                _workflow_settlement_publication,
                _authorization,
                _idempotency,
                mut direct_run,
                mutation_run,
                _attempt_basis,
                _aftermath_causality,
            ) = running.into_parts_for_test();
            let sibling = fork_product(&world, lease.product());
            let sibling = world
                .application
                .select_product_branch_for_test(&phase.borrowed_request_for_test(), &sibling)
                .expect("the exact sibling product remains selectable");
            let (_, sibling_product, sibling_application_basis) = sibling.into_parts_for_test();
            drop(sibling_application_basis);

            let result = direct_run
                .admit_provider_execution_plan(
                    (*world.application.runtime_for_test()).primary_graph_authority_for_test(),
                )
                .expect("the application authorities admit the provider plan")
                .bind_application_product_for_test(sibling_product);
            assert!(result.is_err());
            finish_uncommitted(mutation_run, direct_run, lease);
        })
        .expect("fixture owner admits its advancement");
}

#[test]
fn the_exact_selected_product_binds_at_provider_plan_boundary() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    world
        .application
        .with_host_advancement(|active_phase| {
            let host_custody = active_phase.test_host_request_for_test();
            let phase = &active_phase;

            let running = start(
                phase,
                &world,
                retained_program(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    "product-owner",
                ),
                idempotency(195, 196),
            );
            let (
                _admission,
                lease,
                _provider_attempt,
                _outcome_identity,
                _workflow_settlement_publication,
                _authorization,
                _idempotency,
                mut direct_run,
                mutation_run,
                _attempt_basis,
                _aftermath_causality,
            ) = running.into_parts_for_test();
            let result = direct_run
                .admit_provider_execution_plan(
                    (*world.application.runtime_for_test()).primary_graph_authority_for_test(),
                )
                .expect("the application authorities admit the provider plan")
                .bind_application_product_for_test(lease.product().retained_clone_for_test());
            assert!(result.is_ok());
            drop(result);
            finish_uncommitted(mutation_run, direct_run, lease);
        })
        .expect("fixture owner admits its advancement");
}

#[test]
fn foreign_installed_phase_refuses_provider_readmission_before_read_or_charge() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    use worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test as reports;
    use worth_query_execution::facade::domain_computation::installed_source_reads_on_this_thread_for_test as reads;
    use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
    use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;
    for placement in [
        Placement::Serial,
        Placement::Leased(std::num::NonZeroUsize::MIN),
    ] {
        let previous = place(placement);
        let a =
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let b =
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let scope = live_scope();
        let mut arun = a
            .application
            .with_application_advancement(&scope, |phase| {
                start(
                    &phase,
                    &a,
                    retained_program(&phase, &a, "readmission-a"),
                    idempotency(181, 181),
                )
            })
            .unwrap();
        let mut brun = b
            .application
            .with_application_advancement(&scope, |phase| {
                start(
                    &phase,
                    &b,
                    retained_program(&phase, &b, "readmission-b"),
                    idempotency(182, 182),
                )
            })
            .unwrap();
        reports();
        a.application
            .with_application_advancement(&scope, |phase| {
                let before = reads();
                // Read before the running run is borrowed mutably.
                let product = (*brun.lease_for_test()).product().retained_clone_for_test();
                let plan = (*brun
                    .running_mut_for_test())
                    .admit_provider_execution_plan((*b.application.runtime_for_test()).primary_graph_authority_for_test())
                    .unwrap()
                    .bind_application_product_for_test(product)
                    .unwrap();
                let refusal = match plan.readmit(&phase) {
                    Err(refusal) => refusal,
                    Ok(_) => panic!("foreign phase cannot call B's provider"),
                };
                assert!(matches!(refusal.kind(),
                worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: Resource::ForeignAdvancementPhase, partition_identity: None,
                    policy_ancestor: None,
                }));
                assert_eq!(refusal.counters().provider_calls(), 0);
                assert_eq!(reads(), before);
            })
            .unwrap();
        let refused = reports();
        assert_eq!(refused.len(), 1);

        a.application
            .with_application_advancement(&scope, |phase| {
                let before = reads();
                // Read before the running run is borrowed mutably.
                let product = (*arun.lease_for_test()).product().retained_clone_for_test();
                let session =
                    real_terminal_session(&phase, &a, &mut (*arun.running_mut_for_test()), product);
                assert_eq!(session.counters().provider_calls(), 2);
                // The primary provider's readmission and preparation only mint and
                // check session tokens (provider/session_lifecycle.rs:12-36).
                // Its real consumer counter is the two provider calls, not a read.
                assert_eq!(reads(), before);
                let _ = session.abort();
            })
            .unwrap();
        let (
            _a_admission,
            a_lease,
            _a_attempt,
            _a_outcome,
            _a_settlement,
            _a_authorization,
            _a_idempotency,
            a_running,
            a_mutation_run,
            _a_basis,
            _a_causality,
        ) = arun.into_parts_for_test();
        finish_uncommitted(a_mutation_run, a_running, a_lease);
        let (
            _b_admission,
            b_lease,
            _b_attempt,
            _b_outcome,
            _b_settlement,
            _b_authorization,
            _b_idempotency,
            b_running,
            b_mutation_run,
            _b_basis,
            _b_causality,
        ) = brun.into_parts_for_test();
        finish_uncommitted(b_mutation_run, b_running, b_lease);
        place(previous);
    }
}
