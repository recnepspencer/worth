pub(crate) use {std::collections::BTreeSet};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_relational::facade::branch::RelationalBranchIdentity};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind, worth_relational::facade::inspection::RelationalAuthoritativeAllocationLocator, worth_relational::facade::inspection::RelationalMvccCostScope};
pub(crate) use {worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) fn persistent_node_locators(
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
                == worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootReachabilityStructure
        })
        .map(|allocation| allocation.locator())
        .collect()
}

pub(crate) fn assert_persistent_path_reuse(
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

pub(crate) fn persistent_path_reuse_holds(
    before: &BTreeSet<RelationalAuthoritativeAllocationLocator>,
    after: &BTreeSet<RelationalAuthoritativeAllocationLocator>,
    expected_new_nodes: u64,
) -> bool {
    after.difference(before).count() as u64 == expected_new_nodes
        && before.difference(after).count() as u64 == expected_new_nodes
        && before.intersection(after).count() as u64 == before.len() as u64 - expected_new_nodes
}