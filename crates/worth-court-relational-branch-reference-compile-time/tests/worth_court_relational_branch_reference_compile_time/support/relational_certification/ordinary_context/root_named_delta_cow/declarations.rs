pub(crate) use std::collections::BTreeSet;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_supply_chain_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::PartitionId};
pub(crate) use {worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind, worth_relational::facade::inspection::RelationalBranchSharingCostCounters, worth_relational::facade::inspection::RelationalMvccCostScope};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntime};

pub(crate) fn prove_named_delta_cow(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    branch: &str,
    label: BranchLabel,
    delta: DeltaId,
) {
    let (world, baseline) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork_from_main(execution, &world.runtime, branch);
    fork_from_main(execution, &world.runtime, "cost-sibling");
    let selected = world
        .runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .unwrap();
    let main = world.runtime.main_branch_identity();
    let sibling = world
        .runtime
        .branch_identity(&BranchId("cost-sibling".to_owned()))
        .unwrap();
    let selected_before = world
        .runtime
        .observe_branch_sharing(std::slice::from_ref(&selected))
        .unwrap();
    let before_regions = selected_before
        .region_locators()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let touched = declared_partitions(&world.handles, delta);
    let before_owner_ledger = world
        .runtime
        .inspect_owner_allocation_ledger(std::slice::from_ref(&selected))
        .unwrap();
    let before_owner_locators = before_owner_ledger
        .authoritative_allocations()
        .iter()
        .map(|allocation| allocation.locator())
        .collect::<BTreeSet<_>>();
    let whole_touched_partition_bytes = before_owner_ledger
        .authoritative_allocations()
        .iter()
        .filter(|allocation| {
            allocation.locator().kind() == worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::PartitionPayload
                && allocation
                    .locator()
                    .partition_id()
                    .is_some_and(|partition| touched.contains(&partition))
        })
        .map(|allocation| allocation.authoritative_bytes())
        .sum::<u64>();
    let selected_scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
        &world.runtime,
        vec![selected.clone()],
    );
    let main_scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
        &world.runtime,
        vec![main],
    );
    let sibling_scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
        &world.runtime,
        vec![sibling],
    );
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId(branch.to_owned()),
        &std::collections::BTreeSet::new(),
        delta,
    )
    .expect("the actual branch pre-state admits its named delta");
    commit_supply_chain_delta(
        execution,
        &world.runtime,
        &world.program,
        BranchId(branch.to_owned()),
        delta,
        batch,
    );

    let cost = world.runtime.observe_mvcc_cost(&selected_scope).unwrap();
    let publication = cost.sharing_cost_delta();
    let after_regions = cost
        .sharing()
        .region_locators()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let created = after_regions
        .difference(&before_regions)
        .copied()
        .collect::<BTreeSet<_>>();
    let reused = after_regions.intersection(&before_regions).count() as u64;
    assert_eq!(
        created
            .iter()
            .map(|region| region.partition_id())
            .collect::<BTreeSet<_>>(),
        touched
    );
    assert_eq!(
        publication.publication_touched_region_count,
        touched.len() as u64
    );
    assert_eq!(publication.publication_reused_region_count, reused);
    assert!(publication.copied_truth_bytes > 0);
    assert!(
        publication.copied_truth_bytes < whole_touched_partition_bytes,
        "named delta must not copy every owner in its touched partitions"
    );
    let after_owner_ledger = world
        .runtime
        .inspect_owner_allocation_ledger(std::slice::from_ref(&selected))
        .unwrap();
    let exact_new_bytes = after_owner_ledger
        .authoritative_allocations()
        .iter()
        .filter(|allocation| !before_owner_locators.contains(&allocation.locator()))
        .map(|allocation| allocation.authoritative_bytes())
        .sum::<u64>();
    assert_eq!(
        publication.publication_new_authoritative_bytes,
        exact_new_bytes
    );
    for unrelated in [&main_scope, &sibling_scope] {
        assert_eq!(
            world
                .runtime
                .observe_mvcc_cost(unrelated)
                .unwrap()
                .sharing_cost_delta(),
            worth_relational::facade::inspection::RelationalBranchSharingCostCounters::default()
        );
    }
    let snapshot = snapshot_for_supply_chain_identity(&world.runtime, &selected);
    let observed = observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        &snapshot,
    )
    .unwrap();
    compare(
        &expected_supply_chain_branch(&world.program, label, Some(delta)),
        &observed,
    )
    .unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .expect("the exact semantic proof releases its selected snapshot");
}

pub(crate) fn declared_partitions(
    handles: &SupplyChainSemanticHandles,
    delta: DeltaId,
) -> BTreeSet<PartitionId> {
    let entity = |kind, ordinal| {
        handles.entities[&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(kind, ordinal)]
            .id
            .partition_id
    };
    let relation = |kind, ordinal| {
        handles.relations[&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(kind, ordinal)]
            .id
            .partition_id
    };
    match delta {
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora => [
            entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 0),
            relation(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1),
        ]
        .into_iter()
        .collect(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth => [
            entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 0),
            entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 0),
            relation(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth, 0),
        ]
        .into_iter()
        .collect(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo => [entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, 0)].into_iter().collect(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity => [
            entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal, 1),
            entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 2),
        ]
        .into_iter()
        .collect(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival => [entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 0)].into_iter().collect(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora => [
            entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 0),
            entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection, 0),
        ]
        .into_iter()
        .collect(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall => [
            entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, 1),
            relation(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1),
        ]
        .into_iter()
        .collect(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2 => {
            [entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, 0)].into_iter().collect()
        }
    }
}

pub(crate) fn fork_from_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &str,
) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    runtime
        .fork_branch(execution, BranchId(branch.to_owned()), source)
        .unwrap();
}
