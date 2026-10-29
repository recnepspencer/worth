#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::root_branch_copy_on_write::*;

#[test]
fn phase5_forked_topology_write_copies_only_touched_regions() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork_from_main(execution, &world.runtime, "rewire");
    fork_from_main(execution, &world.runtime, "maintenance");

    let main = world.runtime.main_branch_identity();
    let rewire = branch_identity(&world.runtime, "rewire");
    let sibling = branch_identity(&world.runtime, "maintenance");
    let branches = [main.clone(), rewire.clone(), sibling.clone()];
    let before_all = sharing(&world.runtime, &branches);
    let before_main = region_set(&sharing(&world.runtime, std::slice::from_ref(&main)));
    let before_rewire = region_set(&sharing(&world.runtime, std::slice::from_ref(&rewire)));
    let before_sibling = region_set(&sharing(&world.runtime, std::slice::from_ref(&sibling)));
    assert_eq!(before_rewire, before_main);
    assert_eq!(before_sibling, before_main);
    assert_eq!(before_all.unique_root_count(), 1);
    let call_key = EntityKey::new(EntityKind::PortCall, 1);
    let relation_key = RelationKey::new(RelationKind::CallAtPort, 1);
    let declared_touched_partitions = [
        world.handles.entities[&call_key].id.partition_id,
        world.handles.relations[&relation_key].id.partition_id,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let before_owner_ledger = world
        .runtime
        .inspect_owner_allocation_ledger(std::slice::from_ref(&rewire))
        .expect("storage owners inventory the exact pre-write root");
    let before_owner_locators = before_owner_ledger
        .authoritative_allocations()
        .iter()
        .map(|allocation| allocation.locator())
        .collect::<BTreeSet<_>>();
    let whole_touched_partition_bytes = before_owner_ledger
        .authoritative_allocations()
        .iter()
        .filter(|allocation| {
            allocation.locator().kind() == RelationalAuthoritativeAllocationKind::PartitionPayload
                && allocation
                    .locator()
                    .partition_id()
                    .is_some_and(|id| declared_touched_partitions.contains(&id))
        })
        .map(|allocation| allocation.authoritative_bytes())
        .sum::<u64>();
    assert!(
        declared_touched_partitions.len() < before_rewire.len(),
        "the named record-plus-relation delta must be narrower than the complete world"
    );
    assert_eq!(
        before_all.logical_branch_partition_payload_bytes(),
        before_all.unique_physical_partition_payload_bytes() * before_all.branch_count(),
        "three unchanged branches must account for exactly three logical views of one root"
    );

    let delta = DeltaId::RewireAuroraPortCall;
    let expected = expected_rewire_from_pure_oracle(&world, delta);
    let cost_scope = RelationalMvccCostScope::capture(&world.runtime, vec![rewire.clone()]);
    let main_cost_scope = RelationalMvccCostScope::capture(&world.runtime, vec![main.clone()]);
    let sibling_cost_scope =
        RelationalMvccCostScope::capture(&world.runtime, vec![sibling.clone()]);
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId("rewire".to_owned()),
        &BTreeSet::new(),
        delta,
    )
    .expect("the actual rewire branch pre-state lowers to production intent");
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("rewire".to_owned()),
        batch,
    );

    assert_rewire_matches_oracle(execution, &mut world, &rewire, expected);
    let cost = world
        .runtime
        .observe_mvcc_cost(&cost_scope)
        .expect("the rewire branch owns its publication cost");
    let after_rewire = region_set(cost.sharing());
    let after_main = region_set(&sharing(&world.runtime, std::slice::from_ref(&main)));
    let after_sibling = region_set(&sharing(&world.runtime, std::slice::from_ref(&sibling)));
    let after_all = sharing(&world.runtime, &branches);
    let retired_from_branch = before_rewire
        .difference(&after_rewire)
        .copied()
        .collect::<BTreeSet<_>>();
    let created_for_branch = after_rewire
        .difference(&before_rewire)
        .copied()
        .collect::<BTreeSet<_>>();
    let reused_by_branch = before_rewire
        .intersection(&after_rewire)
        .copied()
        .collect::<BTreeSet<_>>();
    let publication = cost.sharing_cost_delta();

    assert_eq!(after_main, before_main, "the ancestor root must not move");
    assert_eq!(
        after_sibling, before_sibling,
        "an untouched sibling must retain every exact ancestor region locator"
    );
    assert_eq!(
        publication.publication_touched_region_count,
        created_for_branch.len() as u64
    );
    assert_eq!(
        publication.publication_touched_region_count,
        declared_touched_partitions.len() as u64,
        "publication breadth must equal the partitions declared by the semantic handles"
    );
    assert_eq!(
        publication.publication_touched_region_count,
        retired_from_branch.len() as u64
    );
    assert_eq!(
        publication.publication_reused_region_count,
        reused_by_branch.len() as u64
    );
    assert_eq!(
        publication.publication_reused_region_count,
        before_rewire.len() as u64 - declared_touched_partitions.len() as u64,
        "every untouched ancestor partition must be reused"
    );
    assert!(publication.copied_truth_bytes > 0);
    assert!(
        publication.copied_truth_bytes < whole_touched_partition_bytes,
        "selected publication must copy paths, not complete touched partitions"
    );
    assert_eq!(publication.copied_commit_envelopes, 0);
    assert!(publication.publication_new_authoritative_bytes > 0);
    let after_owner_ledger = world
        .runtime
        .inspect_owner_allocation_ledger(std::slice::from_ref(&rewire))
        .expect("storage owners inventory the exact post-write root");
    let exact_new_authoritative_bytes = after_owner_ledger
        .authoritative_allocations()
        .iter()
        .filter(|allocation| !before_owner_locators.contains(&allocation.locator()))
        .map(|allocation| allocation.authoritative_bytes())
        .sum::<u64>();
    assert_eq!(
        publication.publication_new_authoritative_bytes, exact_new_authoritative_bytes,
        "publication accounting must equal the independent owner-ledger allocation delta"
    );
    assert_eq!(
        after_all.unique_physical_authoritative_bytes(),
        before_all.unique_physical_authoritative_bytes()
            + publication.publication_new_authoritative_bytes,
        "the combined physical increase must equal every new authoritative owner allocation"
    );
    let expected_combined_locators = before_main
        .union(&created_for_branch)
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(region_set(&after_all), expected_combined_locators);
    assert_eq!(after_all.unique_root_count(), 2);
    assert_eq!(after_all.unique_canonical_commit_artifacts(), 2);
    assert!(
        after_all.logical_branch_partition_payload_bytes()
            > after_all.unique_physical_partition_payload_bytes(),
        "shared ancestor regions must be counted logically per branch and physically once"
    );
    let rewire_only = cost.sharing();
    assert_eq!(
        rewire_only.logical_branch_partition_payload_bytes(),
        rewire_only.unique_physical_partition_payload_bytes(),
        "one selected branch has no cross-branch physical deduplication"
    );
    for unrelated_scope in [&main_cost_scope, &sibling_cost_scope] {
        assert_eq!(
            world
                .runtime
                .observe_mvcc_cost(unrelated_scope)
                .unwrap()
                .sharing_cost_delta(),
            RelationalBranchSharingCostCounters::default(),
            "each unrelated branch records exactly zero copy/materialization work"
        );
    }
}
