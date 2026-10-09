//! Provider-plan affinity to the exact installed product.
use super::*;
#[test]
fn an_exact_sibling_product_cannot_substitute_at_provider_plan_binding() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;

            let running = start(
                phase,
                &world,
                retained_program(&world, "product-owner"),
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
                .select_product_branch(&sibling)
                .expect("the exact sibling product remains selectable");
            let (_, sibling_product, sibling_application_basis) = sibling.into_parts();
            drop(sibling_application_basis);

            let result = direct_run
                .admit_provider_execution_plan(&world.application.primary_graph_authority)
                .expect("the application authorities admit the provider plan")
                .bind_application_product(sibling_product);
            assert!(result.is_err());
            finish_uncommitted(mutation_run, direct_run, lease);
        })
        .expect("fixture owner admits its advancement");
}

#[test]
fn the_exact_selected_product_binds_at_provider_plan_boundary() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;

            let running = start(
                phase,
                &world,
                retained_program(&world, "product-owner"),
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
                .admit_provider_execution_plan(&world.application.primary_graph_authority)
                .expect("the application authorities admit the provider plan")
                .bind_application_product(lease.product().retained_clone());
            assert!(result.is_ok());
            drop(result);
            finish_uncommitted(mutation_run, direct_run, lease);
        })
        .expect("fixture owner admits its advancement");
}

#[cfg(feature = "test-query-execution-observer")]
#[test]
fn foreign_installed_phase_refuses_provider_readmission_before_read_or_charge() {
    use crate::domain_computation::primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        installed_source_reads_on_this_thread_for_test as reads,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
        WorthQueryManagedComputationResourceDenial as Resource,
    };
    for placement in [
        Placement::Serial,
        Placement::Leased(std::num::NonZeroUsize::MIN),
    ] {
        let previous = place(placement);
        let a =
            crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
                true,
            );
        let b =
            crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
                true,
            );
        let scope = live_scope();
        let mut arun = a
            .application
            .with_application_advancement(&scope, |phase| {
                start(
                    &phase,
                    &a,
                    retained_program(&a, "readmission-a"),
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
                    retained_program(&b, "readmission-b"),
                    idempotency(182, 182),
                )
            })
            .unwrap();
        reports();
        a.application
            .with_application_advancement(&scope, |phase| {
                let before = reads();
                let plan = brun
                    .running
                    .admit_provider_execution_plan(&b.application.primary_graph_authority)
                    .unwrap()
                    .bind_application_product(brun.lease.product().retained_clone())
                    .unwrap();
                let refusal = match plan.readmit(&phase) {
                    Err(refusal) => refusal,
                    Ok(_) => panic!("foreign phase cannot call B's provider"),
                };
                assert!(matches!(refusal.kind(),
                crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: Resource::ForeignAdvancementPhase, partition_identity: None,
                    policy_ancestor: None,
                }));
                assert_eq!(refusal.counters().provider_calls(), 0);
                assert_eq!(reads(), before);
            })
            .unwrap();
        let refused = reports();
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].as_ref().unwrap().charged_work(), 0);
        a.application
            .with_application_advancement(&scope, |phase| {
                let before = reads();
                let session = real_terminal_session(
                    &phase,
                    &a,
                    &mut arun.running,
                    arun.lease.product().retained_clone(),
                );
                assert_eq!(session.counters().provider_calls(), 2);
                // The primary provider's readmission and preparation only mint and
                // check session tokens (provider/session_lifecycle.rs:12-36).
                // Its real consumer counter is the two provider calls, not a read.
                assert_eq!(reads(), before);
                let _ = session.abort();
            })
            .unwrap();
        finish_uncommitted(arun.mutation_run, arun.running, arun.lease);
        finish_uncommitted(brun.mutation_run, brun.running, brun.lease);
        place(previous);
    }
}
