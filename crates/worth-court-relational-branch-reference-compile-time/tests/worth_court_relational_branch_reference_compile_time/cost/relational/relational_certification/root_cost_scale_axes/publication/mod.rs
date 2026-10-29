#[allow(unused_imports)]
use crate::relational_certification::reference_context::root_cost_scale_axes::*;
pub(crate) use std::collections::BTreeSet;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch_with_result, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::footprint_batch::lower_cargo_footprint_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::inspection::RelationalMvccCostScope};
pub(crate) use {worth_relational::facade::transactions::WorkerIntentBatch};

#[test]
fn production_write_footprints_report_exact_records_and_local_publication_cost() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let small_copy = single_record_copy_cost(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::standard();
    let (world, expected) = certified_supply_chain_world(execution, scale);
    assert_oracle_matches(execution, &world, &expected);
    let mut physical_costs = Vec::new();
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
        let identity = world.runtime.branch_identity(&branch).unwrap();
        let scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
            &world.runtime,
            vec![identity.clone()],
        );
        let before = world
            .runtime
            .inspect_owner_allocation_ledger(std::slice::from_ref(&identity))
            .unwrap();
        let old_allocations: BTreeSet<_> = before
            .authoritative_allocations()
            .iter()
            .map(|allocation| allocation.locator())
            .collect();
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
        let cost = world.runtime.observe_mvcc_cost(&scope).unwrap();
        let delta = cost.sharing_cost_delta();
        assert_eq!(delta.transaction_validation_attempts, 1);
        assert_eq!(delta.candidate_preparations, 1);
        assert_eq!(delta.publication_attempts, 1);
        assert_eq!(delta.branch_population_scans, 0);
        assert_eq!(delta.copied_commit_envelopes, 0);
        let after = world
            .runtime
            .inspect_owner_allocation_ledger(std::slice::from_ref(&identity))
            .unwrap();
        let independently_new_bytes: u64 = after
            .authoritative_allocations()
            .iter()
            .filter(|allocation| !old_allocations.contains(&allocation.locator()))
            .map(|allocation| allocation.authoritative_bytes())
            .sum();
        assert_eq!(
            delta.publication_new_authoritative_bytes,
            independently_new_bytes
        );
        assert!(after
            .authoritative_allocations()
            .iter()
            .any(|allocation| old_allocations.contains(&allocation.locator())));
        physical_costs.push((
            footprint,
            delta.publication_touched_region_count,
            delta.publication_persistent_index_path_nodes,
            delta.copied_truth_bytes,
            delta.publication_new_authoritative_bytes,
        ));
    }
    assert!(physical_costs.windows(2).all(|pair| pair[0].1 == pair[1].1));
    assert!(physical_costs.windows(2).all(|pair| pair[0].2 == pair[1].2));
    assert!(physical_costs.windows(2).all(|pair| pair[0].3 < pair[1].3));
    assert!(physical_costs.windows(2).all(|pair| pair[0].4 < pair[1].4));
    assert!(physical_costs[0].3 > 0);
    // The same one-record edit at 32x cargo population may add index levels,
    // but must not approach linear copying. Fourfold headroom covers the
    // additional binary page-index levels (4 pages -> 128 pages).
    assert_eq!(scale.cargo_lots, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court().cargo_lots * 32);
    assert!(
        physical_costs[0].3 <= small_copy * 4,
        "one-record copy grew from {small_copy} to {} at 32x population",
        physical_costs[0].3
    );
    assert!(
        physical_costs[2].3 < physical_costs[0].3 * physical_costs[2].0 as u64,
        "one publication must share copied paths across its changed records"
    );
}

#[test]
fn selected_publication_cost_is_flat_through_ordinary_retained_histories() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_flat_history_samples(execution, &[1, 64, 1_024]);
}
