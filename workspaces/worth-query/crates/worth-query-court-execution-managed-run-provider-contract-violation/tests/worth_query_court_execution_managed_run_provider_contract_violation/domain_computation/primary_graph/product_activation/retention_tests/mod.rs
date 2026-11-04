use worth_query_execution::facade::domain_computation::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::activation::denial::WorthQueryProductActivationDenial;
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::activation::registry::WorthQueryProductActivationRegistry;
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;

#[test]
fn activation_capacity_follows_reserved_and_retired_gate_allocations() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    let limit = test_product_world_resources(
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    )
    .budgets_for_test()
    .live_product_branches();
    let registry = WorthQueryProductActivationRegistry::new(limit).unwrap();
    registry
        .reserve_for_test()
        .unwrap()
        .commit_for_test(product.observation_for_test());
    let held_gate = registry.gate_for_test(product.branch_identity()).unwrap();
    let mut pending = (1..limit.get().expect("explicit finite branch policy"))
        .map(|_| registry.reserve_for_test().unwrap())
        .collect::<Vec<_>>();
    assert!(matches!(
        registry.reserve_for_test(),
        Err(WorthQueryProductActivationDenial::CapacityExhausted)
    ));

    // Removing membership cannot reclaim backing still held by an operation.
    assert!(registry.release_for_test(
        product.branch_identity(),
        product.observation_for_test().lifecycle_incarnation()
    ));
    assert!(matches!(
        registry.gate_for_test(product.branch_identity()),
        Err(WorthQueryProductActivationDenial::UnknownProductBranch)
    ));
    assert!(matches!(
        registry.reserve_for_test(),
        Err(WorthQueryProductActivationDenial::CapacityExhausted)
    ));
    drop(held_gate);
    let replacement = registry.reserve_for_test().unwrap();
    assert!(matches!(
        registry.reserve_for_test(),
        Err(WorthQueryProductActivationDenial::CapacityExhausted)
    ));
    drop(replacement);

    // Abandoning an admitted creation frees its slot before any World effect.
    pending.pop();
    let first = registry.reserve_for_test().unwrap();
    let second = registry.reserve_for_test().unwrap();
    assert!(matches!(
        registry.reserve_for_test(),
        Err(WorthQueryProductActivationDenial::CapacityExhausted)
    ));
    drop((first, second, pending));
    for _ in 0..(limit.get().expect("explicit finite branch policy") * 2) {
        drop(registry.reserve_for_test().unwrap());
    }
}

#[test]
fn foreign_product_is_denied_by_owner_affinity_before_local_coordination() {
    let local = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let foreign = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selection = foreign
        .application
        .product_runtime()
        .admit_product_branch_for_test(
            foreign
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    assert!(matches!(
        (*(*local
            .application
            .runtime_for_test())
            .product_runtime_for_test())
            .admit_product_branch_for_test(selection.branch_identity()),
        Err(
            worth_query_execution::facade::domain_computation::WorthQueryProductBranchAdmissionDenial::Refused(
                AdmissionRefusal::ForeignOwner
            )
        )
    ));
}
