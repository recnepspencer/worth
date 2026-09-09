use crate::domain_computation::execution_runtime::product_world::{
    activation::{WorthQueryProductActivationDenial, WorthQueryProductActivationRegistry},
    installed_budgets,
};
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;

#[test]
fn activation_capacity_follows_reserved_and_retired_gate_allocations() {
    let world = installed_authorization_world(true);
    let product = world.application.admit_current_product_branch().unwrap();
    let limit = installed_budgets().live_product_branches();
    let registry = WorthQueryProductActivationRegistry::new(limit).unwrap();
    registry.reserve().unwrap().commit(product.observation());
    let held_gate = registry.gate(product.branch_identity()).unwrap();
    let mut pending = (1..limit.get())
        .map(|_| registry.reserve().unwrap())
        .collect::<Vec<_>>();
    assert!(matches!(
        registry.reserve(),
        Err(WorthQueryProductActivationDenial::CapacityExhausted)
    ));

    // Removing membership cannot reclaim backing still held by an operation.
    assert!(registry.release(
        product.branch_identity(),
        product.observation().lifecycle_incarnation()
    ));
    assert!(matches!(
        registry.gate(product.branch_identity()),
        Err(WorthQueryProductActivationDenial::UnknownProductBranch)
    ));
    assert!(matches!(
        registry.reserve(),
        Err(WorthQueryProductActivationDenial::CapacityExhausted)
    ));
    drop(held_gate);
    let replacement = registry.reserve().unwrap();
    assert!(matches!(
        registry.reserve(),
        Err(WorthQueryProductActivationDenial::CapacityExhausted)
    ));
    drop(replacement);

    // Abandoning an admitted creation frees its slot before any World effect.
    pending.pop();
    let first = registry.reserve().unwrap();
    let second = registry.reserve().unwrap();
    assert!(matches!(
        registry.reserve(),
        Err(WorthQueryProductActivationDenial::CapacityExhausted)
    ));
    drop((first, second, pending));
    for _ in 0..(limit.get() * 2) {
        drop(registry.reserve().unwrap());
    }
}

#[test]
fn foreign_product_is_denied_by_owner_affinity_before_local_coordination() {
    let local = installed_authorization_world(true);
    let foreign = installed_authorization_world(true);
    let selection = foreign.application.admit_current_product_branch().unwrap();
    assert!(matches!(
        local
            .application
            .product_runtime
            .admit_product_branch(selection.branch_identity()),
        Err(crate::basis::WorthQueryProductBranchAdmissionDenial::ForeignOwner)
    ));
}
