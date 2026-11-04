//! Provider-plan affinity to the exact installed product.
use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::affinity_tests::*;
#[test]
fn an_exact_sibling_product_cannot_substitute_at_provider_plan_binding() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
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
            let WorthQueryRunningApplicationCommit {
                admission: _admission,
                lease,
                provider_attempt: _provider_attempt,
                authorization: _authorization,
                idempotency: _idempotency,
                running: mut direct_run,
                mutation_run,
                attempt_basis: _attempt_basis,
                aftermath_causality: _aftermath_causality,
                outcome_identity: _outcome_identity,
                workflow_settlement_publication: _workflow_settlement_publication,
            } = running;
            let sibling = fork_product(&world, lease.product());
            let sibling = world
                .application
                .select_product_branch_for_test(&phase.borrowed_request_for_test(), &sibling)
                .expect("the exact sibling product remains selectable");
            let (_, sibling_product, sibling_application_basis) = sibling.into_parts_for_test();
            drop(sibling_application_basis);

            let result = direct_run
                .admit_provider_execution_plan(&(*world.application.primary_graph_authority_for_test()))
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
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
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
            let WorthQueryRunningApplicationCommit {
                admission: _admission,
                lease,
                provider_attempt: _provider_attempt,
                authorization: _authorization,
                idempotency: _idempotency,
                running: mut direct_run,
                mutation_run,
                attempt_basis: _attempt_basis,
                aftermath_causality: _aftermath_causality,
                outcome_identity: _outcome_identity,
                workflow_settlement_publication: _workflow_settlement_publication,
            } = running;
            let result = direct_run
                .admit_provider_execution_plan(&(*world.application.primary_graph_authority_for_test()))
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
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
                true,
            );
        let b =
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
                true,
            );
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
                let plan = (*brun
                    .running_mut_for_test())
                    .admit_provider_execution_plan(&(*b.application.primary_graph_authority_for_test()))
                    .unwrap()
                    .bind_application_product_for_test((*brun.lease_for_test()).product().retained_clone_for_test())
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
                let session = real_terminal_session(
                    &phase,
                    &a,
                    &mut (*arun.running_mut_for_test()),
                    (*arun.lease_for_test()).product().retained_clone_for_test(),
                );
                assert_eq!(session.counters().provider_calls(), 2);
                // The primary provider's readmission and preparation only mint and
                // check session tokens (provider/session_lifecycle.rs:12-36).
                // Its real consumer counter is the two provider calls, not a read.
                assert_eq!(reads(), before);
                let _ = session.abort();
            })
            .unwrap();
        finish_uncommitted(arun.mutation_run_into_for_test(), arun.running_into_for_test(), (*arun.lease_for_test()));
        finish_uncommitted(brun.mutation_run_into_for_test(), brun.running_into_for_test(), (*brun.lease_for_test()));
        place(previous);
    }
}
