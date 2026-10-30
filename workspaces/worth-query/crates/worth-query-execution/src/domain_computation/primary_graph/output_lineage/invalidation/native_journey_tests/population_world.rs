//! Finite native-write allowance for the positive bulk-population fixture.
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world_with_product_resources, AuthorizationWorld,
};

pub(super) fn install(population: usize) -> AuthorizationWorld {
    let work = u64::try_from(population)
        .unwrap()
        .checked_mul(32_000)
        .and_then(|work| work.checked_add(8_000_000))
        .expect("native population fixture work is finite");
    let resources =
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources();
    let declared = resources.execution_policy();
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        declared.posture(),
        declared.determinism(),
        worth_foundational::ExecutionBudget::new(declared.max_workers(), None, Some(work)),
    );
    installed_authorization_world_with_product_resources(resources.with_execution_policy(policy))
}
