use std::num::NonZeroUsize;
use std::panic::{catch_unwind, AssertUnwindSafe};

use super::settlement_failures::assert_idempotency_refuses_unpublished;
use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};

#[test]
fn world_unwind_retains_and_releases_the_exact_query_idempotency_slot() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "post-relational-unwind",
    );
    let retry_while_retained = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "post-relational-unwind",
    );
    let retry_after_cleanup = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "post-relational-unwind",
    );
    let before_inspections =
        crate::domain_computation::primary_graph::provider::unwind_recovery_inspection_count();
    world
        .application
        .product_runtime()
        .operation_control()
        .panic_before_product_compare_once();

    let unwind = catch_unwind(AssertUnwindSafe(|| {
        world
            .application
            .compare_and_commit_application(program, idempotency(0xD1, 0xD1))
    }));
    let isolated = unwind.expect("the application boundary isolates provider unwind");
    assert!(
        matches!(
            isolated,
            WorthQueryApplicationCommitOutcome::Indeterminate(_)
        ),
        "provider unwind is typed without claiming a product commit: {isolated:?}",
    );
    assert_eq!(
        crate::domain_computation::primary_graph::provider::unwind_recovery_inspection_count(),
        before_inspections + 1,
        "only the unwind guard inspects its exact World recovery identity",
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .unpublished_idempotency_count(),
        1,
    );

    let page = world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(1).unwrap())
        .unwrap();
    let [row] = page.rows() else {
        panic!("World must retain the exact owner movement abandoned by unwind")
    };
    let recovery = world
        .application
        .readmit_product_publication_recovery(row.handle())
        .unwrap();
    let before_retry_inspections =
        crate::domain_computation::primary_graph::provider::unwind_recovery_inspection_count();
    assert_idempotency_refuses_unpublished(
        world
            .application
            .compare_and_commit_application(retry_while_retained, idempotency(0xD1, 0xD1)),
    );
    assert_eq!(
        crate::domain_computation::primary_graph::provider::unwind_recovery_inspection_count(),
        before_retry_inspections,
        "ordinary idempotency resolution must not inspect World recovery",
    );
    recovery.release_obligations(0).unwrap();
    assert_eq!(
        world
            .application
            .primary_provider
            .unpublished_idempotency_count(),
        0,
        "cleanup releases Query's exact recovery-indexed slot",
    );

    let after = world
        .application
        .compare_and_commit_application(retry_after_cleanup, idempotency(0xD1, 0xD1));
    let WorthQueryApplicationCommitOutcome::Denied(denial) = after else {
        panic!("the unpublished Relational movement leaves the product basis stale: {after:?}")
    };
    assert_ne!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::Idempotency,
        "cleanup removes the retained idempotency denial before fresh admission",
    );
}
