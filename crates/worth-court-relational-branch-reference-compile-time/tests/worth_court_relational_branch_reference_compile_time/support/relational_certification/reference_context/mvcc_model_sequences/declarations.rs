pub(crate) use std::collections::BTreeSet;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_supply_chain_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_relational::facade::branch::RelationalBranchIdentity};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::inspection::RelationalBranchSharingCostCounters, worth_relational::facade::inspection::RelationalMvccCostScope};

pub(crate) fn assert_all_sibling_semantics(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld,
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

pub(crate) fn assert_v1_sibling_progressed_after_v2_transition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld,
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
    assert_eq!(observed.schema, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1);
    compare(
        &expected_supply_chain_branch(
            &world.program,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire,
            Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall),
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

pub(crate) fn assert_main_remains_operating(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld,
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
        &expected_supply_chain_branch(&world.program, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating, None),
        &observed,
    )
    .expect("all sibling publications leave main semantic truth unchanged");
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
}

pub(crate) fn all_identities(
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

pub(crate) fn fork_from_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    branch: BranchId,
) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    runtime.fork_branch(execution, branch, source).unwrap();
}

pub(crate) fn branch_id(delta: DeltaId) -> BranchId {
    BranchId(
        match delta {
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora => "storm",
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth => "maintenance",
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo => "medical-hold",
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity => "southpoint-expansion",
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival => "competing-arrival",
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora => "inspection",
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall => "rewire",
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2 => "hazard-v2",
        }
        .to_owned(),
    )
}
