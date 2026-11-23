use crate::branch::RelationalBranchBasisDenial as Denial;
use crate::tests::support::runtime_with_test_schema;

#[test]
fn retained_basis_extension_requires_actual_current_history_owner_after_reset() {
    let mut runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let (_, before) = runtime.observe_branch(&identity).unwrap();
    // Keep the actual prior owner alive: a runtime-id-only check would still
    // be able to acquire through that old binding and wrongly succeed.
    let prior_owner = runtime.history.retention_owner.clone();
    let foreign = runtime_with_test_schema();
    assert!(matches!(
        foreign.retain_component_basis(&before),
        Err(Denial::ForeignRuntime { .. })
    ));
    drop(runtime.retain_component_basis(&before).unwrap());
    runtime.set_retention_capacity_for_test(64, 64);
    assert_eq!(runtime.main_branch_identity(), identity);
    assert_eq!(
        before.identity().runtime_instance_id(),
        runtime.runtime_instance_id()
    );
    assert!(matches!(
        runtime.retain_component_basis(&before),
        Err(Denial::UnavailableRetainedTarget)
    ));
    let (_, current) = runtime.observe_branch(&identity).unwrap();
    assert_eq!(
        current.descriptor(),
        before.descriptor(),
        "matching metadata cannot replace actual owner custody"
    );
    drop(runtime.retain_component_basis(&current).unwrap());
    drop(prior_owner);
}
