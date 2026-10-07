use super::{
    admitted_operation, admitted_program, authenticated_principal, idempotency,
    installed_authorization_world, live_scope, resolved_account,
};

#[test]
fn admitted_ordinary_recovery_uses_the_original_commit_and_exact_world_successor() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let commits = || {
        world
            .application
            .primary_provider
            .graph
            .with_open_runtime(|runtime| runtime.history().immutable_commit_count())
    };
    let baseline = commits();
    let program = admitted_program(&world, &principal, &account, &request, "recovered");
    world.fail_next_durable_append();
    let WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) = world
        .application
        .compare_and_commit_application(program, idempotency(191, 191))
    else {
        panic!("the fault must retain the exact World partial");
    };
    let recovery = partial.into_recovery();
    recovery.continue_owner_settlement().unwrap();
    let fresh = admitted_operation(&world, &principal, &account, &request);
    let outcome = world
        .application
        .recover_admitted_unpublished_application(&recovery, &fresh, idempotency(191, 191))
        .expect("the same authorized intent can adopt its settled World partial");
    let crate::domain_computation::primary_graph::WorthQueryManagedApplicationRecoveryOutcome::Performed(performed) = outcome else {
        panic!("settled ordinary recovery must finish the original application");
    };
    let (read, publication_failure, cleanup_failure) = performed.into_parts();
    assert!(publication_failure.is_none());
    assert!(cleanup_failure.is_none());
    assert!(matches!(read.unwrap().into_resolution(), crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(_)));
    assert_eq!(
        commits(),
        baseline + 1,
        "adoption cannot rerun the Relational handler"
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .unpublished_idempotency_count(),
        0
    );
}
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};
use crate::domain_computation::{
    WorthQueryProductUnpublishedRecoveryReleaseDenial,
    WorthQueryProductUnpublishedRecoveryReleaseFailure,
};
use std::num::NonZeroUsize;
use worth_runtime_world::facade::{
    ProductUnpublishedCause, ProductUnpublishedNextAction, RuntimeWorldRecoveryDenial,
};

#[test]
fn product_unpublished_settlement_repairs_owner_only_and_cleanup_is_exact() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
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
            .with_open_runtime(|runtime| runtime.history().immutable_commit_count())
    };
    let baseline = commits();
    let program = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "performed-before-durable-fault",
    );
    world.fail_next_durable_append();
    let outcome = world
        .application
        .compare_and_commit_application(program, idempotency(91, 91));
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

#[test]
fn dropped_partial_is_rediscovered_and_idempotent_retry_cannot_promote_owner_rows() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
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
            .with_open_runtime(|runtime| runtime.history().immutable_commit_count())
    };
    let baseline = commits();
    let program = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "drop-then-idempotent-retry",
    );
    let retry_before = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "drop-then-idempotent-retry",
    );
    let retry_after = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "drop-then-idempotent-retry",
    );
    world.fail_next_durable_append();
    let outcome = world
        .application
        .compare_and_commit_application(program, idempotency(96, 96));
    let WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) = outcome else {
        panic!("durable append failure must retain the unpublished owner occurrence: {outcome:?}");
    };
    drop(partial);
    let page = world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(1).unwrap())
        .unwrap();
    assert!(page.examined() <= 1);
    let [row] = page.rows() else {
        panic!("one dropped application partial must remain in World's catalog")
    };
    let recovery = world
        .application
        .readmit_product_publication_recovery(row.handle())
        .unwrap();
    assert!(recovery.inspect().unwrap().relational_requires_settlement());
    assert_idempotency_refuses_unpublished(
        world
            .application
            .compare_and_commit_application(retry_before, idempotency(96, 96)),
    );
    recovery.continue_owner_settlement().unwrap();
    assert_idempotency_refuses_unpublished(
        world
            .application
            .compare_and_commit_application(retry_after, idempotency(96, 96)),
    );
    let current = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    assert_eq!(current.selected_commit(), selected.selected_commit());
    assert_eq!(commits(), baseline + 1);
    let receipt = world
        .application
        .release_product_publication_recovery(recovery, 0)
        .unwrap();
    assert_eq!(receipt.retired_component_count(), 0);
    assert_eq!(
        world
            .application
            .primary_provider
            .unpublished_idempotency_count(),
        0,
        "rediscovered cleanup releases the same bounded tombstone",
    );
}

pub(super) fn assert_idempotency_refuses_unpublished(outcome: WorthQueryApplicationCommitOutcome) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("owner-local idempotency row without original product publication must remain ineligible: {outcome:?}");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::Idempotency
    );
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
}
