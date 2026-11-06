use std::time::Duration;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use crate::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_operation;
pub use crate::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
pub use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::decision_currentness::assert_changed_decision;
pub use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::decision_currentness::assert_product_basis_stale;
pub fn idempotency(key: u8, intent: u8) -> WorthQueryApplicationIdempotencyBinding {
    WorthQueryApplicationIdempotencyBinding::new_for_test([key; 32], [intent; 32])
}
pub fn authenticated_principal(
    phase: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal<
    crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
    crate::domain_computation::primary_graph::tests::fixture::Principal,
    u64,
> {
    let external = world.authenticate("alice", Duration::from_secs(60), request);
    let execution = phase
        .execution_request_for_for_test(world.application.product_runtime())
        .expect("fixture lends its installed phase");
    world
        .application
        .select_product_branch_for_test(
            &execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &phase.phase_for_application_for_test(&world.application),
            &world.binding,
            &external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}
pub fn resolved_account(
    phase: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    status: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity<
    crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
    crate::domain_computation::primary_graph::tests::fixture::Account,
> {
    let execution = phase
        .execution_request_for_for_test(world.application.product_runtime())
        .expect("fixture lends its installed phase");
    world
        .application
        .select_product_branch_for_test(
            &execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase.phase_for_application_for_test(&world.application),
            AccountStatus::reference(),
            status.to_string(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

#[path = "preimage_evidence/mod.rs"]
pub mod preimage_evidence;
#[path = "program_fixture/mod.rs"]
pub mod program_fixture;
#[path = "recovery_custody/mod.rs"]
pub mod recovery_custody;
#[path = "unwind_custody/mod.rs"]
pub mod unwind_custody;
