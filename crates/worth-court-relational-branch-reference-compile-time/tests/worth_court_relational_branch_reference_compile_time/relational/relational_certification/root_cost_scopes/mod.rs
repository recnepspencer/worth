#[allow(unused_imports)] use crate::relational_certification::reference_context::root_cost_scopes::*;
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world};
pub(crate) use {worth_relational::facade::history::BranchId};

#[test]
fn cost_scope_excludes_prior_target_work_and_later_sibling_work() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, expected) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &expected);
    fork(execution, &world.runtime, "before-capture");
    let observed_branch = world
        .runtime
        .branch_identity(&BranchId("before-capture".to_owned()))
        .expect("the owner issues the target identity after fork");
    let scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
        &world.runtime,
        vec![observed_branch],
    );
    fork(execution, &world.runtime, "after-capture");
    let observation = world.runtime.observe_mvcc_cost(&scope).unwrap();
    assert_eq!(observation.sharing_cost_delta().shared_root_acquisitions, 0);
}
