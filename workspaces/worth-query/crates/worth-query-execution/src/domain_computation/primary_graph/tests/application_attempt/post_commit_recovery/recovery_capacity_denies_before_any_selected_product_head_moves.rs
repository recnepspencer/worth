//! recovery capacity denies before any selected product head moves.
use super::*;

#[test]
fn recovery_capacity_denies_before_any_selected_product_head_moves() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let resources =
        WorthQueryApplicationQueryResourceProfile::bounded(5_120, 2_048, usize::MAX, 2).unwrap();
    let world = installed_authorization_world_with_resource_profile(resources);
    let source = world.application.current_world();
    let sibling_a_branch = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(source)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .expect("the first real sibling product must publish")
        })
        .expect("the fixture read owns its installed host call");
    let sibling_b_branch = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(source)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .expect("the second real sibling product must publish")
        })
        .expect("the fixture read owns its installed host call");
    let root = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let sibling_a = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(sibling_a_branch)
                .select(&phase)
                .expect("the first sibling remains selectable")
        })
        .expect("the fixture read owns its installed host call");
    let sibling_b = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(sibling_b_branch)
                .select(&phase)
                .expect("the second sibling remains selectable")
        })
        .expect("the fixture read owns its installed host call");
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let world_head_before = sibling_b.product().selected_commit().clone();
    let relational_head_before = sibling_b
        .product()
        .observation()
        .basis()
        .relational_basis()
        .descriptor()
        .clone();
    let relational_branch = relational_head_before.branch_id().clone();
    let publication_cost_scope =
        world
            .application
            .primary_provider
            .graph
            .with_runtime(request_execution, |runtime| {
                let identity = runtime
                    .branch_identity(&relational_branch)
                    .expect("the sibling's exact Relational branch remains installed");
                worth_relational::facade::inspection::RelationalMvccCostScope::capture(
                    runtime,
                    vec![identity],
                )
            });
    let provider_publications_before = world
        .application
        .primary_provider
        .published_application_commit_count();

    let root_slot = world
        .application
        .primary_provider
        .reserve_application_publication_recovery(root.product().observation())
        .expect("total minus one recovery slot remains admissible");
    let sibling_a_slot = world
        .application
        .primary_provider
        .reserve_application_publication_recovery(sibling_a.product().observation())
        .expect("the final installed recovery slot remains admissible");
    let denied_program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_on_selected(
                &phase,
                &world,
                &sibling_b,
                &principal,
                &account,
                &request,
                "recovery-capacity-denied",
            )
        })
        .expect("the fixture read owns its installed host call");
    let denied = world
        .application
        .on_branch(sibling_b_branch)
        .transaction()
        .apply(WorthQueryAdmittedChange::new(
            denied_program,
            idempotency(200, 201),
        ))
        .commit()
        .expect("the public product token remains admitted");
    assert!(matches!(
        denied,
        WorthQueryApplicationCommitOutcome::Aborted
    ));
    assert_eq!(
        world
            .application
            .primary_provider
            .pending_application_publication_count_for_test(),
        2,
        "the denied attempt must not acquire or disturb either retained slot"
    );
    let after_denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(sibling_b_branch)
                .select(&phase)
                .expect("capacity denial leaves the sibling selectable")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(after_denial.product().selected_commit(), &world_head_before);
    assert_eq!(
        after_denial
            .product()
            .observation()
            .basis()
            .relational_basis()
            .descriptor(),
        &relational_head_before,
        "recovery capacity must deny before the Relational head moves"
    );
    let publication_cost =
        world
            .application
            .primary_provider
            .graph
            .with_runtime(request_execution, |runtime| {
                runtime
                    .observe_mvcc_counters(&publication_cost_scope)
                    .expect("the exact sibling cost scope remains observable")
            });
    assert_eq!(
        publication_cost.sharing_cost_delta().publication_attempts,
        0,
        "capacity denial must precede Relational publication owner contact"
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        provider_publications_before,
        "capacity denial must publish no application causality"
    );

    drop(root_slot);
    let fresh = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_on_selected(
                &phase,
                &world,
                &after_denial,
                &principal,
                &account,
                &request,
                "recovery-capacity-denied",
            )
        })
        .expect("the fixture read owns its installed host call");
    let committed = world
        .application
        .on_branch(sibling_b_branch)
        .transaction()
        .apply(WorthQueryAdmittedChange::new(fresh, idempotency(200, 201)))
        .commit()
        .expect("the public product token remains admitted")
        .require_committed()
        .expect("releasing one slot must admit and commit a fresh exact twin");
    assert_ne!(
        committed.committed_product_publication().composite_commit(),
        &world_head_before
    );
    drop(committed);
    drop(sibling_a_slot);
    assert_eq!(
        world
            .application
            .primary_provider
            .pending_application_publication_count_for_test(),
        0
    );
    drop(root);
    drop(sibling_a);
    drop(sibling_b);
    drop(after_denial);
    world
        .application
        .with_host_advancement(|phase| {
            assert!(world
                .application
                .on_branch(sibling_a_branch)
                .close(&phase,)
                .unwrap()
                .is_complete())
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            assert!(world
                .application
                .on_branch(sibling_b_branch)
                .close(&phase,)
                .unwrap()
                .is_complete())
        })
        .expect("the fixture read owns its installed host call");
}
