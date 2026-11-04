use crate::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use crate::domain_computation::primary_graph::tests::application_attempt::idempotency;
use crate::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use crate::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use crate::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use std::num::NonZeroUsize;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub fn assert_idempotency_refuses_unpublished(outcome: WorthQueryApplicationCommitOutcome) {
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
pub fn retained_recovery_after_application_unwind(
    world: &crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    commit: impl FnOnce() -> WorthQueryApplicationCommitOutcome,
) -> worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecovery {
    world
        .application
        .product_runtime()
        .operation_control_for_test()
        .panic_before_product_compare_once_for_test();
    let outcome = catch_unwind(AssertUnwindSafe(commit))
        .expect("the application boundary isolates the actual World interruption");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationCommitOutcome::Indeterminate(_)
        ),
        "real post-effect unwind retains its typed unresolved outcome: {outcome:?}"
    );
    let page = world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::MIN)
        .unwrap();
    let [row] = page.rows() else {
        panic!("the actual interrupted owner effect appears on this bounded page")
    };
    let recovery = world
        .application
        .readmit_product_publication_recovery(row.handle())
        .unwrap();
    assert_eq!(recovery.inspect().unwrap().owner_effect_count(), 1);
    recovery
}
