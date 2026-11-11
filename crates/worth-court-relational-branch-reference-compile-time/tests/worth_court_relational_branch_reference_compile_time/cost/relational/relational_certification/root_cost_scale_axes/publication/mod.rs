#[allow(unused_imports)]
use crate::relational_certification::reference_context::root_cost_scale_axes::*;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch_with_result, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::footprint_batch::lower_cargo_footprint_batch};
pub(crate) use {worth_relational::facade::history::BranchId};

#[test]
fn production_write_footprints_publish_the_declared_record_counts() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::standard();
    let (world, expected) = certified_supply_chain_world(execution, scale);
    assert_oracle_matches(execution, &world, &expected);
    for footprint in [1, 64, 4_096] {
        let branch = BranchId(format!("footprint-{footprint}"));
        let (_, source) = world
            .runtime
            .observe_fork_source(&BranchId("main".to_owned()))
            .unwrap();
        world
            .runtime
            .fork_branch(execution, branch.clone(), source)
            .unwrap();
        let batch = lower_cargo_footprint_batch(&world.handles, scale, footprint);
        let committed = commit_branch_batch_with_result(execution, &world.runtime, branch, batch);
        assert_eq!(
            committed.publication_summary().unwrap().patch_record_count,
            footprint
        );
        world
            .runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .unwrap();
    }
}

#[test]
fn selected_publication_cost_is_flat_through_ordinary_retained_histories() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_flat_history_samples(execution, &[1, 64, 1_024]);
}
