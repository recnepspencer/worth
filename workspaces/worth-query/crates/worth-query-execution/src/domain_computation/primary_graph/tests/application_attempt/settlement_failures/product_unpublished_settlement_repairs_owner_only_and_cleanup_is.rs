//! product unpublished settlement repairs owner only and cleanup is exact.
use super::*;

#[test]
fn product_unpublished_settlement_repairs_owner_only_and_cleanup_is_exact() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
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
    let selected = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let commits = || {
        world
            .application
            .primary_provider
            .graph
            .with_runtime(request_execution, |runtime| {
                runtime.history().immutable_commit_count()
            })
    };
    let baseline = commits();
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "performed-before-durable-fault",
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .fail_next_durable_append_for_test(request_execution);
    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(91, 91),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) = outcome else {
        panic!("owner movement without settled product publication must retain partial custody: {outcome:?}");
    };
    assert_eq!(partial.cause(), ProductUnpublishedCause::SettlementPending);
    assert_eq!(partial.owner_effect_count(), 1);
    assert!(partial.relational_requires_settlement());
    assert_eq!(
        partial.expected_product().selected_commit(),
        selected.selected_commit()
    );
    assert_eq!(commits(), baseline + 1);
    assert_eq!(
        world
            .application
            .primary_provider
            .unpublished_idempotency_count(),
        1,
        "the unpublished owner result consumes one bounded tombstone",
    );
    let recovery = partial.into_recovery();
    let foreign = installed_authorization_world(true);
    assert!(matches!(
        foreign
            .application
            .readmit_product_publication_recovery(recovery.record_handle()),
        Err(RuntimeWorldRecoveryDenial::ForeignHandle)
    ));
    let failure = world
        .application
        .release_product_publication_recovery(recovery, 0)
        .expect_err("unsettled owner effects must remain recoverable");
    let WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failure) = failure else {
        panic!("unsettled owner effects cannot become cleanup custody")
    };
    assert_eq!(
        failure.denial(),
        WorthQueryProductUnpublishedRecoveryReleaseDenial::World(
            RuntimeWorldRecoveryDenial::SettlementRequired
        )
    );
    let recovery = failure.into_recovery();
    let held_view = recovery.inspect().unwrap();
    assert!(matches!(
        recovery.continue_owner_settlement(),
        Err(RuntimeWorldRecoveryDenial::CallerCapabilityLive)
    ));
    drop(held_view);
    let actions = recovery.continue_owner_settlement().unwrap();
    assert!(!actions
        .actions()
        .contains(&ProductUnpublishedNextAction::SettleOwnerEffects));
    assert!(actions
        .actions()
        .contains(&ProductUnpublishedNextAction::StartFreshCompositePublication));
    assert_eq!(recovery.continue_owner_settlement().unwrap(), actions);
    let settled = recovery.inspect().unwrap();
    assert!(!settled.relational_requires_settlement());
    assert_eq!(settled.owner_effect_count(), 1);
    assert!(settled.live_obligation_count() > 0);
    drop(settled);
    let current = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    assert_eq!(
        current.selected_commit(),
        selected.selected_commit(),
        "settlement cannot publish product truth"
    );
    assert_eq!(
        commits(),
        baseline + 1,
        "settlement must not rerun the application"
    );
    let receipt = world
        .application
        .release_product_publication_recovery(recovery, 0)
        .unwrap();
    assert_eq!(receipt.retired_component_count(), 0);
    assert!(world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(1).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(
        world
            .application
            .primary_provider
            .unpublished_idempotency_count(),
        0,
        "terminal World cleanup releases the exact tombstone capacity",
    );
}
