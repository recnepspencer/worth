//! Real pending custody races cross-branch lookup through the admitted resolver.
//! The owner-private command marker isolates store/coordination evidence from domain derivation.
use super::*;
use crate::domain_computation::primary_graph::{
    provider::CorrectionIdempotencyObservation as Observed,
    WorthQueryApplicationIdempotencyResolution,
};

#[test]
fn pending_correction_cutover_never_looks_unseen_on_another_branch() {
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request()))
        .unwrap();
    let _delivery = world
        .application
        .primary_provider
        .observe_application_commit_causality(selected.product());
    let request = live_scope();
    let sibling = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let sibling_observation = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(sibling)
                .select(&phase)
                .unwrap()
                .product()
                .observation()
                .clone()
        })
        .unwrap();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&phase.test_host_request(), &world, &request)
        })
        .unwrap();
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&phase.test_host_request(), &world, "open", &request)
        })
        .unwrap();
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "pending-correction",
                Some("one-correction-effect"),
            )
        })
        .unwrap();
    let binding = idempotency(0xC1, 0xC2).bind_correction_command(&[0xC3; 32], &[0xC4; 32]);
    let held = world
        .application
        .hold_post_commit_snapshot_failure_for_test()
        .unwrap();
    let outcome = world.application.compare_and_commit_application(
        program,
        binding,
        ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationCommitOutcome::Indeterminate(_)
        ),
        "the held real post-commit boundary must leave pending custody: {outcome:?}"
    );
    let provider = &world.application.primary_provider;
    assert!(
        matches!(provider.observe_correction_key(&sibling_observation, binding.key_identity()),
        Observed::Pending(Some(recorded)) if recorded.key_identity() == binding.key_identity())
    );
    drop(held);
    let barrier = std::sync::Barrier::new(2);
    let (resolved, observed) = std::thread::scope(|threads| {
        let resolver = threads.spawn(|| {
            barrier.wait();
            resolve(&world, &request, binding)
        });
        let observer = threads.spawn(|| {
            barrier.wait();
            provider.observe_correction_key(&sibling_observation, binding.key_identity())
        });
        (resolver.join().unwrap(), observer.join().unwrap())
    });
    let WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) = resolved else {
        panic!("the fresh admitted resolver must finish the original correction custody");
    };
    match observed {
        Observed::Pending(Some(recorded)) => {
            assert_eq!(recorded, receipt.authority_binding().idempotency_binding())
        }
        Observed::Completed(recorded, committed) => {
            assert_eq!(recorded, receipt.authority_binding().idempotency_binding());
            assert_eq!(committed.commit_reference(), receipt.commit_reference());
        }
        _ => panic!("cross-branch lookup must retain exact pending or completed evidence"),
    }
    assert_eq!(provider.pending_application_publication_count_for_test(), 0);
    assert_eq!(receipt.emitted_effect_count(), 1);
    assert_eq!(
        provider
            .committed_application_emissions(receipt.committed_product_publication())
            .len(),
        1
    );
    let snapshots = || {
        let host = crate::bridge_host_request::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        provider.graph.with_runtime(&execution, |runtime| {
            runtime
                .storage_access()
                .storage_stats(&execution)
                .unwrap()
                .published_snapshot_handle_count
        })
    };
    let baseline = snapshots();
    let replayed = resolve(&world, &request, binding);
    let WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(replayed) = replayed else {
        panic!("the completed command remains exactly replayable");
    };
    assert!(replayed.is_same_authoritative_commit(&receipt));
    assert_eq!(
        snapshots(),
        baseline,
        "equivalent resolution releases its temporary snapshots"
    );
}

fn resolve(
    world: &crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    binding: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryApplicationIdempotencyResolution {
    world
        .application
        .with_host_advancement(|phase| {
            let principal = authenticated_principal(&phase.test_host_request(), world, request);
            let account = resolved_account(
                &phase.test_host_request(),
                world,
                "pending-correction",
                request,
            );
            let admission = super::super::program_fixture::admitted_operation(
                &phase, world, &principal, &account, request,
            );
            world
                .application
                .resolve_admitted_application_idempotency(&phase, &admission, binding)
                .unwrap()
                .into_resolution()
        })
        .unwrap()
}
