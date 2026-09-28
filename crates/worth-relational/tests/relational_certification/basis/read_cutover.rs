use std::collections::BTreeSet;

use super::world::supply_chain::{
    certified_supply_chain_world, commit_branch_batch, fork_supply_chain_branch_from_main,
    lower_supply_chain_production_delta, observe_supply_chain_observation, DeltaId, EntityRecord,
    SupplyChainScale,
};
use worth_relational::facade::branch::RelationalBranchBasisDenial;
use worth_relational::facade::history::{BranchId, RelationalMergeBranchBasisDenial};

#[test]
fn history_and_visibility_read_the_observation_selected_root() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let branch_id = BranchId("storm".to_owned());
    fork_supply_chain_branch_from_main(&world.runtime, branch_id.clone());
    let identity = world.runtime.branch_identity(&branch_id).unwrap();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let observation = basis.observation();
    let voyage_status = |observed: &_| {
        let EntityRecord::Voyage(voyage) = &observe_supply_chain_observation(
            &world.program,
            &world.handles,
            &world.runtime,
            observed,
        )
        .unwrap()
        .entities[&world.handles.aurora_voyage().semantic] else {
            panic!("Aurora remains a voyage in the admitted root");
        };
        voyage.status
    };
    let status_before = voyage_status(&observation);
    let original_head = world
        .runtime
        .history()
        .branch_head_for_observation(&observation)
        .unwrap()
        .expect("baseline has one canonical head");

    let batch = lower_supply_chain_production_delta(
        &world.runtime,
        &world.program,
        &world.handles,
        &branch_id,
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .unwrap();
    commit_branch_batch(&world.runtime, branch_id, batch);

    let (_, current_basis) = world.runtime.observe_branch(&identity).unwrap();
    let current_observation = current_basis.observation();
    let current_head = world
        .runtime
        .history()
        .branch_head_for_observation(&current_observation)
        .unwrap()
        .unwrap();
    assert_ne!(current_head.commit_id, original_head.commit_id);
    assert_eq!(
        world
            .runtime
            .history()
            .branch_head_for_observation(&observation)
            .unwrap()
            .unwrap()
            .commit_id,
        original_head.commit_id
    );
    assert_eq!(voyage_status(&observation), status_before);
    assert_ne!(voyage_status(&current_observation), status_before);
}

#[test]
fn merge_history_resolves_from_two_exact_observations() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let main_identity = world.runtime.main_branch_identity();
    let (_, main_basis) = world.runtime.observe_branch(&main_identity).unwrap();
    let main_observation = main_basis.observation();
    let main_head = world
        .runtime
        .history()
        .branch_head_for_observation(&main_observation)
        .unwrap()
        .unwrap();

    let storm = BranchId("storm".to_owned());
    fork_supply_chain_branch_from_main(&world.runtime, storm.clone());
    let batch = lower_supply_chain_production_delta(
        &world.runtime,
        &world.program,
        &world.handles,
        &storm,
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .unwrap();
    commit_branch_batch(&world.runtime, storm.clone(), batch);
    let storm_identity = world.runtime.branch_identity(&storm).unwrap();
    let (_, storm_basis) = world.runtime.observe_branch(&storm_identity).unwrap();
    let storm_observation = storm_basis.observation();
    let storm_head = world
        .runtime
        .history()
        .branch_head_for_observation(&storm_observation)
        .unwrap()
        .unwrap();

    let merge_basis = world
        .runtime
        .history()
        .merge_branch_basis_for_observations(&storm_observation, &main_observation)
        .unwrap();

    assert_eq!(merge_basis.source_head().commit_id, storm_head.commit_id);
    assert_eq!(merge_basis.target_head().commit_id, main_head.commit_id);
    assert_eq!(
        merge_basis.merge_base().commit().commit_id,
        main_head.commit_id
    );
}

#[test]
fn merge_history_preserves_source_and_target_observation_denials() {
    let (owner, _) = certified_supply_chain_world(SupplyChainScale::court());
    let (foreign, _) = certified_supply_chain_world(SupplyChainScale::court());
    let owner_identity = owner.runtime.main_branch_identity();
    let foreign_identity = foreign.runtime.main_branch_identity();
    let (_, owner_basis) = owner.runtime.observe_branch(&owner_identity).unwrap();
    let (_, foreign_basis) = foreign.runtime.observe_branch(&foreign_identity).unwrap();
    let owner_observation = owner_basis.observation();
    let foreign_observation = foreign_basis.observation();

    let source_denial = owner
        .runtime
        .history()
        .merge_branch_basis_for_observations(&foreign_observation, &owner_observation)
        .expect_err("foreign source observation must be denied as the source");
    assert!(matches!(
        source_denial,
        RelationalMergeBranchBasisDenial::SourceObservationDenied(
            RelationalBranchBasisDenial::ForeignRuntime { .. }
        )
    ));

    let target_denial = owner
        .runtime
        .history()
        .merge_branch_basis_for_observations(&owner_observation, &foreign_observation)
        .expect_err("foreign target observation must be denied as the target");
    assert!(matches!(
        target_denial,
        RelationalMergeBranchBasisDenial::TargetObservationDenied(
            RelationalBranchBasisDenial::ForeignRuntime { .. }
        )
    ));
}
