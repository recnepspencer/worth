use std::time::Duration;
use crate::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use crate::domain_computation::primary_graph::tests::fixture::program_roster::installed_program_support;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::program_required_binding::ProgramRequiredInput;
use crate::domain_computation::primary_graph::tests::fixture::ProgramRequiredOperation;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitTerminalKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use crate::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_operation;
pub use crate::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use crate::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_mutation_free_program;
use crate::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program_on_selected;
use crate::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program_with_emit;
use crate::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program_with_expected_status;
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
            world.application.product_runtime().default_branch_for_test(),
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
            world.application.product_runtime().default_branch_for_test(),
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
