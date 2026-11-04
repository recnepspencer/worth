//! Finite native-write allowance for the positive bulk-population fixture.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world_with_product_resources;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;

pub fn install(population: usize) -> AuthorizationWorld {
    const NATIVE_AUTHORING_ALLOWANCE_PER_ENTITY: u64 = 1_000_000;
    let work = u64::try_from(population)
        .unwrap()
        .checked_mul(NATIVE_AUTHORING_ALLOWANCE_PER_ENTITY)
        .and_then(|work| work.checked_add(8_000_000))
        .expect("native population fixture work is finite");
    let resources =
        worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let declared = resources.execution_policy();
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        declared.posture(),
        declared.determinism(),
        worth_foundational::ExecutionBudget::new(declared.max_workers(), None, Some(work)),
    );
    installed_authorization_world_with_product_resources(resources.with_execution_policy(policy))
}
