use std::collections::BTreeSet;

use super::invariant_oracle_expectations::expected_supply_chain_branch;
use super::world::supply_chain::{
    assert_oracle_matches, certified_supply_chain_world, commit_supply_chain_delta, compare,
    lower_supply_chain_production_delta, observe_supply_chain_snapshot,
    snapshot_for_supply_chain_identity, BranchLabel, DeltaId, EntityKey, EntityKind, RelationKey,
    RelationKind, SupplyChainScale, SupplyChainSemanticHandles,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::inspection::{
    RelationalAuthoritativeAllocationKind, RelationalBranchSharingCostCounters,
    RelationalMvccCostScope,
};
use worth_relational::facade::runtime::RelationalRuntime;

#[test]
fn every_supply_chain_delta_shares_payloads_within_its_declared_partition_footprint() {
    let scenarios = [
        ("storm", BranchLabel::Storm, DeltaId::StormRerouteAurora),
        (
            "maintenance",
            BranchLabel::Maintenance,
            DeltaId::MaintainAtlasBerth,
        ),
        (
            "medical-hold",
            BranchLabel::MedicalHold,
            DeltaId::HoldMedicalCargo,
        ),
        (
            "southpoint-expansion",
            BranchLabel::SouthpointExpansion,
            DeltaId::ExpandSouthpointCapacity,
        ),
        (
            "competing-arrival",
            BranchLabel::CompetingArrival,
            DeltaId::CompetingAuroraArrival,
        ),
        (
            "inspection",
            BranchLabel::Inspection,
            DeltaId::RetireAtlasWhileInspectingAurora,
        ),
        ("rewire", BranchLabel::Rewire, DeltaId::RewireAuroraPortCall),
        (
            "hazard-v2",
            BranchLabel::HazardV2,
            DeltaId::AdoptHazardClassificationV2,
        ),
    ];
    for (branch, label, delta) in scenarios {
        prove_named_delta_cow(branch, label, delta);
    }
}

fn prove_named_delta_cow(branch: &str, label: BranchLabel, delta: DeltaId) {
    let (world, baseline) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &baseline);
    fork_from_main(&world.runtime, branch);
    fork_from_main(&world.runtime, "cost-sibling");
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
            allocation.locator().kind() == RelationalAuthoritativeAllocationKind::PartitionPayload
                && allocation
                    .locator()
                    .partition_id()
                    .is_some_and(|partition| touched.contains(&partition))
        })
        .map(|allocation| allocation.authoritative_bytes())
        .sum::<u64>();
    let selected_scope = RelationalMvccCostScope::capture(&world.runtime, vec![selected.clone()]);
    let main_scope = RelationalMvccCostScope::capture(&world.runtime, vec![main]);
    let sibling_scope = RelationalMvccCostScope::capture(&world.runtime, vec![sibling]);
    let batch = lower_supply_chain_production_delta(
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId(branch.to_owned()),
        &BTreeSet::new(),
        delta,
    )
    .expect("the actual branch pre-state admits its named delta");
    commit_supply_chain_delta(
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
            RelationalBranchSharingCostCounters::default()
        );
    }
    let snapshot = snapshot_for_supply_chain_identity(&world.runtime, &selected);
    let observed = observe_supply_chain_snapshot(
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

fn declared_partitions(
    handles: &SupplyChainSemanticHandles,
    delta: DeltaId,
) -> BTreeSet<PartitionId> {
    let entity = |kind, ordinal| {
        handles.entities[&EntityKey::new(kind, ordinal)]
            .id
            .partition_id
    };
    let relation = |kind, ordinal| {
        handles.relations[&RelationKey::new(kind, ordinal)]
            .id
            .partition_id
    };
    match delta {
        DeltaId::StormRerouteAurora => [
            entity(EntityKind::Voyage, 0),
            relation(RelationKind::CallAtPort, 1),
        ]
        .into_iter()
        .collect(),
        DeltaId::MaintainAtlasBerth => [
            entity(EntityKind::Berth, 0),
            entity(EntityKind::Voyage, 0),
            relation(RelationKind::VesselAssignedToBerth, 0),
        ]
        .into_iter()
        .collect(),
        DeltaId::HoldMedicalCargo => [entity(EntityKind::CargoLot, 0)].into_iter().collect(),
        DeltaId::ExpandSouthpointCapacity => [
            entity(EntityKind::Terminal, 1),
            entity(EntityKind::Berth, 2),
        ]
        .into_iter()
        .collect(),
        DeltaId::CompetingAuroraArrival => [entity(EntityKind::Voyage, 0)].into_iter().collect(),
        DeltaId::RetireAtlasWhileInspectingAurora => [
            entity(EntityKind::Berth, 0),
            entity(EntityKind::Inspection, 0),
        ]
        .into_iter()
        .collect(),
        DeltaId::RewireAuroraPortCall => [
            entity(EntityKind::PortCall, 1),
            relation(RelationKind::CallAtPort, 1),
        ]
        .into_iter()
        .collect(),
        DeltaId::AdoptHazardClassificationV2 => {
            [entity(EntityKind::CargoLot, 0)].into_iter().collect()
        }
    }
}

fn fork_from_main(runtime: &RelationalRuntime, branch: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    runtime
        .fork_branch(BranchId(branch.to_owned()), source)
        .unwrap();
}
