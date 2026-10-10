use std::collections::BTreeSet;

use super::invariant_oracle_expectations::expected_supply_chain_branch;
use super::world::supply_chain::{
    assert_oracle_matches, certified_supply_chain_world, commit_supply_chain_delta, compare,
    lower_supply_chain_production_delta, observe_supply_chain_snapshot, BranchLabel, DeltaId,
    SchemaVersion, SupplyChainScale,
};
use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::inspection::{
    RelationalBranchSharingCostCounters, RelationalMvccCostScope,
};

#[test]
fn all_declared_supply_chain_deltas_match_oracle_and_isolate_cost() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);

    let execution_order = [
        DeltaId::StormRerouteAurora,
        DeltaId::MaintainAtlasBerth,
        DeltaId::HoldMedicalCargo,
        DeltaId::ExpandSouthpointCapacity,
        DeltaId::CompetingAuroraArrival,
        DeltaId::RetireAtlasWhileInspectingAurora,
        DeltaId::AdoptHazardClassificationV2,
        DeltaId::RewireAuroraPortCall,
    ];
    let scenarios = execution_order.map(|delta| (branch_id(delta), delta));
    for (branch, _) in &scenarios {
        fork_from_main(&world.runtime, branch.clone());
    }
    let identities = all_identities(&world.runtime, &scenarios);
    let shared = world.runtime.observe_branch_sharing(&identities).unwrap();
    assert_eq!(shared.branch_count(), 9);
    assert_eq!(shared.unique_root_count(), 1);
    assert_eq!(shared.fork_materialized_authoritative_bytes(), 0);
    assert_eq!(shared.copied_commit_envelopes(), 0);
    let rewire_before = world
        .runtime
        .branch_reference_state(&BranchId("rewire".to_owned()))
        .unwrap();
    let mut committed_deltas = Vec::new();

    for (branch, delta) in scenarios.iter().cloned() {
        let references_before = scenarios
            .iter()
            .map(|(candidate, _)| {
                world
                    .runtime
                    .branch_reference_state(candidate)
                    .expect("every sibling has an owner reference")
            })
            .collect::<Vec<_>>();
        let selected = world.runtime.branch_identity(&branch).unwrap();
        let selected_scope =
            RelationalMvccCostScope::capture(&world.runtime, vec![selected.clone()]);
        let unrelated = identities
            .iter()
            .filter(|identity| *identity != &selected)
            .cloned()
            .collect::<Vec<_>>();
        let unrelated_scope = RelationalMvccCostScope::capture(&world.runtime, unrelated);

        let batch = lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &branch,
            &BTreeSet::new(),
            delta,
        )
        .expect("the production lowerer accepts the selected branch's actual state");
        let committed =
            commit_supply_chain_delta(&world.runtime, &world.program, branch.clone(), delta, batch);
        committed_deltas.push(delta);
        for (ordinal, (candidate, _)) in scenarios.iter().enumerate() {
            let after = world.runtime.branch_reference_state(candidate).unwrap();
            if candidate == &branch {
                assert_ne!(
                    after, references_before[ordinal],
                    "the selected branch reference must move"
                );
            } else {
                assert_eq!(
                    after, references_before[ordinal],
                    "a sibling reference must not move with the selected publication"
                );
            }
        }

        assert_eq!(
            committed.schema_transition_summary().is_some(),
            delta == DeltaId::AdoptHazardClassificationV2,
            "only the declared hazard boundary carries schema-transition truth"
        );
        let observed = observe_supply_chain_snapshot(
            execution,
            &world.program,
            &world.handles.for_snapshot(committed.snapshot.clone()),
            &world.runtime,
            &committed.snapshot,
        )
        .expect("the performed branch root is observable through the public exact snapshot");
        compare(
            &expected_supply_chain_branch(&world.program, delta.branch(), Some(delta)),
            &observed,
        )
        .unwrap_or_else(|mismatch| {
            panic!("production delta {delta:?} diverged from the independent oracle: {mismatch:?}")
        });
        world
            .runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .expect("the exact commit snapshot closes once comparison completes");

        let cost = world.runtime.observe_mvcc_cost(&selected_scope).unwrap();
        let delta_cost = cost.sharing_cost_delta();
        assert_eq!(delta_cost.transaction_validation_attempts, 1);
        assert_eq!(delta_cost.candidate_preparations, 1);
        assert_eq!(delta_cost.publication_attempts, 1);
        assert_eq!(delta_cost.branch_population_scans, 0);
        assert_eq!(delta_cost.copied_commit_envelopes, 0);
        assert!(delta_cost.publication_touched_region_count > 0);

        let unrelated_cost = world.runtime.observe_mvcc_cost(&unrelated_scope).unwrap();
        assert_eq!(
            unrelated_cost.sharing_cost_delta(),
            RelationalBranchSharingCostCounters::default(),
            "a branch-local delta must charge no synchronous work to siblings"
        );
        assert_eq!(unrelated_cost.branch_cell_contacts(), 0);
        assert_all_sibling_semantics(execution, &mut world, &scenarios, &committed_deltas);
    }

    assert_v1_sibling_progressed_after_v2_transition(execution, &mut world, &rewire_before);

    let final_sharing = world.runtime.observe_branch_sharing(&identities).unwrap();
    assert_eq!(final_sharing.unique_root_count(), 9);
    assert_eq!(final_sharing.unique_canonical_commit_artifacts(), 9);
    assert_eq!(final_sharing.fork_materialized_authoritative_bytes(), 0);
    assert_eq!(final_sharing.copied_commit_envelopes(), 0);
    assert_main_remains_operating(execution, &mut world);
}

fn assert_all_sibling_semantics(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut super::world::supply_chain::ProductionSeededSupplyChainWorld,
    scenarios: &[(BranchId, DeltaId); 8],
    committed: &[DeltaId],
) {
    for (branch, delta) in scenarios {
        let identity = world.runtime.branch_identity(branch).unwrap();
        let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
        let snapshot = world
            .runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let observed = observe_supply_chain_snapshot(
            execution,
            &world.program,
            &world.handles.for_snapshot(snapshot.clone()),
            &world.runtime,
            &snapshot,
        )
        .unwrap();
        let applied = committed.contains(delta).then_some(*delta);
        compare(
            &expected_supply_chain_branch(&world.program, delta.branch(), applied),
            &observed,
        )
        .unwrap_or_else(|mismatch| {
            panic!("sibling {branch:?} diverged after trace {committed:?}: {mismatch:?}")
        });
        world
            .runtime
            .snapshots()
            .release_snapshot(&snapshot)
            .unwrap();
    }
    assert_main_remains_operating(execution, world);
}

fn assert_v1_sibling_progressed_after_v2_transition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut super::world::supply_chain::ProductionSeededSupplyChainWorld,
    before: &worth_relational::facade::branch::RelationalBranchReferenceState,
) {
    let branch = BranchId("rewire".to_owned());
    let after = world.runtime.branch_reference_state(&branch).unwrap();
    assert_ne!(
        after, *before,
        "the nonempty V1 rewire commit must move its root"
    );
    let identity = world.runtime.branch_identity(&branch).unwrap();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let observed = observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        &snapshot,
    )
    .unwrap();
    assert_eq!(observed.schema, SchemaVersion::V1);
    compare(
        &expected_supply_chain_branch(
            &world.program,
            BranchLabel::Rewire,
            Some(DeltaId::RewireAuroraPortCall),
        ),
        &observed,
    )
    .expect("a retained V1 sibling remains independently publishable after a V2 branch commit");
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
}

fn assert_main_remains_operating(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut super::world::supply_chain::ProductionSeededSupplyChainWorld,
) {
    let identity = world.runtime.main_branch_identity();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let observed = observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        &snapshot,
    )
    .unwrap();
    compare(
        &expected_supply_chain_branch(&world.program, BranchLabel::Operating, None),
        &observed,
    )
    .expect("all sibling publications leave main semantic truth unchanged");
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
}

fn all_identities(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    scenarios: &[(BranchId, DeltaId); 8],
) -> Vec<RelationalBranchIdentity> {
    std::iter::once(runtime.main_branch_identity())
        .chain(
            scenarios
                .iter()
                .map(|(branch, _)| runtime.branch_identity(branch).unwrap()),
        )
        .collect()
}

fn fork_from_main(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    branch: BranchId,
) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    runtime.fork_branch(branch, source).unwrap();
}

fn branch_id(delta: DeltaId) -> BranchId {
    BranchId(
        match delta {
            DeltaId::StormRerouteAurora => "storm",
            DeltaId::MaintainAtlasBerth => "maintenance",
            DeltaId::HoldMedicalCargo => "medical-hold",
            DeltaId::ExpandSouthpointCapacity => "southpoint-expansion",
            DeltaId::CompetingAuroraArrival => "competing-arrival",
            DeltaId::RetireAtlasWhileInspectingAurora => "inspection",
            DeltaId::RewireAuroraPortCall => "rewire",
            DeltaId::AdoptHazardClassificationV2 => "hazard-v2",
        }
        .to_owned(),
    )
}
