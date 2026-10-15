use std::collections::BTreeSet;

use super::world::supply_chain::{assert_oracle_matches, certified_supply_chain_world};
use super::world::supply_chain::{
    commit_branch_batch, lower_supply_chain_production_delta, DeltaId, SupplyChainScale,
};
use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::inspection::{
    RelationalAuthoritativeAllocationKind, RelationalAuthoritativeAllocationLocator,
    RelationalMvccCostScope,
};
use worth_relational::facade::transactions::WorkerIntentBatch;

#[test]
fn phase5_persistent_radix_paths_retain_untouched_owner_allocations() {
    let host_request = crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    const EXPECTED_TOUCHED_REGIONS: u64 = 1;
    const RADIX_NODES_PER_PATH: u64 = 33;
    const EXPECTED_NEW_RADIX_NODES: u64 = EXPECTED_TOUCHED_REGIONS * RADIX_NODES_PER_PATH;

    let (world, expected) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &expected);
    let branch_id = BranchId("storm".to_owned());
    let (_, source) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main is an admitted fork source");
    world
        .runtime
        .fork_branch(branch_id.clone(), source)
        .expect("branch retains the baseline persistent root");
    let identity = world
        .runtime
        .branch_identity(&branch_id)
        .expect("fork identity is owner issued");
    let baseline_nodes = persistent_node_locators(&world.runtime, &identity);
    let mutation_scope = RelationalMvccCostScope::capture(&world.runtime, vec![identity.clone()]);
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &branch_id,
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .expect("storm delta lowers through production intent");
    commit_branch_batch(execution, &world.runtime, branch_id.clone(), batch);

    let after_mutation = persistent_node_locators(&world.runtime, &identity);
    let mutation_cost = world
        .runtime
        .observe_mvcc_cost(&mutation_scope)
        .expect("selected branch reports exact publication work")
        .sharing_cost_delta();
    assert_eq!(
        mutation_cost.publication_touched_region_count, EXPECTED_TOUCHED_REGIONS,
        "Court collocates the two storm writes in one authoritative partition"
    );
    assert_eq!(
        mutation_cost.publication_persistent_index_path_nodes, EXPECTED_NEW_RADIX_NODES,
        "the fixed 32-bit radix allocates exactly one 33-node path per touched partition"
    );
    assert_persistent_path_reuse(&baseline_nodes, &after_mutation, EXPECTED_NEW_RADIX_NODES);

    let reused = baseline_nodes
        .intersection(&after_mutation)
        .next()
        .copied()
        .expect("the mutation leaves at least one untouched persistent subtree");
    let mut dropped_ancestor_mutant = after_mutation.clone();
    assert!(dropped_ancestor_mutant.remove(&reused));
    assert!(
        !persistent_path_reuse_holds(
            &baseline_nodes,
            &dropped_ancestor_mutant,
            mutation_cost.publication_persistent_index_path_nodes,
        ),
        "discarding one untouched ancestor allocation must turn the locator oracle red"
    );

    let before_noop = after_mutation;
    let noop_scope = RelationalMvccCostScope::capture(&world.runtime, vec![identity.clone()]);
    commit_branch_batch(
        execution,
        &world.runtime,
        branch_id,
        WorkerIntentBatch::new("phase5-persistent-path-noop"),
    );
    let after_noop = persistent_node_locators(&world.runtime, &identity);
    let noop_cost = world
        .runtime
        .observe_mvcc_cost(&noop_scope)
        .expect("no-op publication remains inspectable")
        .sharing_cost_delta();
    assert_eq!(noop_cost.publication_touched_region_count, 0);
    assert_eq!(noop_cost.publication_persistent_index_path_nodes, 0);
    assert_eq!(
        after_noop, before_noop,
        "no-op publication reuses every radix allocation"
    );
}

fn persistent_node_locators(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    identity: &RelationalBranchIdentity,
) -> BTreeSet<RelationalAuthoritativeAllocationLocator> {
    runtime
        .inspect_owner_allocation_ledger(std::slice::from_ref(identity))
        .expect("owner ledger accepts its exact branch identity")
        .authoritative_allocations()
        .iter()
        .filter(|allocation| {
            allocation.locator().kind()
                == RelationalAuthoritativeAllocationKind::RootReachabilityStructure
        })
        .map(|allocation| allocation.locator())
        .collect()
}

fn assert_persistent_path_reuse(
    before: &BTreeSet<RelationalAuthoritativeAllocationLocator>,
    after: &BTreeSet<RelationalAuthoritativeAllocationLocator>,
    expected_new_nodes: u64,
) {
    assert!(persistent_path_reuse_holds(
        before,
        after,
        expected_new_nodes
    ));
    assert_eq!(after.difference(before).count() as u64, expected_new_nodes);
    assert_eq!(before.difference(after).count() as u64, expected_new_nodes);
    assert_eq!(
        before.intersection(after).count() as u64,
        before.len() as u64 - expected_new_nodes,
        "every node outside the copied path survives by its exact owner locator"
    );
}

fn persistent_path_reuse_holds(
    before: &BTreeSet<RelationalAuthoritativeAllocationLocator>,
    after: &BTreeSet<RelationalAuthoritativeAllocationLocator>,
    expected_new_nodes: u64,
) -> bool {
    after.difference(before).count() as u64 == expected_new_nodes
        && before.difference(after).count() as u64 == expected_new_nodes
        && before.intersection(after).count() as u64 == before.len() as u64 - expected_new_nodes
}
