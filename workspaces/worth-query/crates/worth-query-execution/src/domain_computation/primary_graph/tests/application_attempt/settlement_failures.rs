use super::{
    admitted_operation, admitted_program, authenticated_principal, idempotency,
    installed_authorization_world, live_scope, resolved_account,
};

#[test]
fn admitted_ordinary_recovery_uses_the_original_commit_and_exact_world_successor() {
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
            admitted_program(&phase, &world, &principal, &account, &request, "recovered")
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .fail_next_durable_append_for_test(request_execution);
    let WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) =
        world.application.compare_and_commit_application(
            program,
            idempotency(191, 191),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the fault must retain the exact World partial");
    };
    let recovery = partial.into_recovery();
    recovery.continue_owner_settlement().unwrap();
    let fresh = world
        .application
        .with_host_advancement(|phase| {
            admitted_operation(&phase, &world, &principal, &account, &request)
        })
        .expect("the fixture read owns its installed host call");
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

#[path = "settlement_failures/product_unpublished_settlement_repairs_owner_only_and_cleanup_is.rs"]
mod product_unpublished_settlement_repairs_owner_only_and_cleanup_is;

#[test]
fn dropped_partial_is_rediscovered_and_idempotent_retry_cannot_promote_owner_rows() {
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
                "drop-then-idempotent-retry",
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry_before = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "drop-then-idempotent-retry",
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry_after = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "drop-then-idempotent-retry",
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .fail_next_durable_append_for_test(request_execution);
    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(96, 96),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
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
    assert_idempotency_refuses_unpublished(world.application.compare_and_commit_application(
        retry_before,
        idempotency(96, 96),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ));
    recovery.continue_owner_settlement().unwrap();
    assert_idempotency_refuses_unpublished(world.application.compare_and_commit_application(
        retry_after,
        idempotency(96, 96),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ));
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
