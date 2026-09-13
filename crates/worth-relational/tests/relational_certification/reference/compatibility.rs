use super::reference_attempt_evidence::{
    assert_denial_left_no_reference_residue, capture_reference_evidence,
};
use super::world::supply_chain::{
    certified_supply_chain_world, head_for_supply_chain_identity,
    snapshot_for_supply_chain_identity, SupplyChainScale,
};
use worth_relational::facade::history::BranchId;

#[test]
fn admitted_observation_reads_do_not_move_branch_cells() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let before = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId("compat".to_owned()),
        world.commit.commit_id,
    );
    let generation_before = before
        .source_state
        .as_ref()
        .expect("main cell")
        .observation()
        .generation()
        .get();
    let truth_before = before
        .source_state
        .as_ref()
        .expect("main cell")
        .truth_version()
        .as_u64();

    let identity = world.runtime.main_branch_identity();
    let head = head_for_supply_chain_identity(&world.runtime, &identity);
    let snapshot = snapshot_for_supply_chain_identity(&world.runtime, &identity);
    assert_eq!(snapshot.version_id(), head.version_id);
    let replay_version = head.version_id;
    assert!(world
        .runtime
        .history_authority()
        .retain_version_for_replay(replay_version));

    let after = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId("compat".to_owned()),
        world.commit.commit_id,
    );
    assert_denial_left_no_reference_residue(&before, &after);
    assert_eq!(
        after
            .source_state
            .as_ref()
            .expect("main cell after compatibility reads")
            .observation()
            .generation()
            .get(),
        generation_before
    );
    assert_eq!(
        after
            .source_state
            .as_ref()
            .expect("main cell after compatibility reads")
            .truth_version()
            .as_u64(),
        truth_before
    );
}

#[test]
fn publication_cannot_mint_a_missing_branch_cell() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    assert!(world
        .runtime
        .branch_identity(&BranchId("ghost".to_owned()))
        .is_err());
    let identity = world.runtime.main_branch_identity();
    let options = world
        .runtime
        .admit_branch_basis(&identity)
        .expect("main remains owner-admissible");
    let before = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId("ghost".to_owned()),
        world.commit.commit_id,
    );
    let mut transaction = world
        .runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    transaction
        .push_batch(
            worth_relational::facade::transactions::WorkerIntentBatch::new("cannot-create-ghost"),
        )
        .unwrap();
    transaction
        .commit(&world.runtime)
        .expect("main publication still advances the admitted main cell");
    let after = capture_reference_evidence(
        &world.runtime,
        &BranchId("main".to_owned()),
        &BranchId("ghost".to_owned()),
        world.commit.commit_id,
    );
    assert!(after.target_state.is_none());
    assert!(world
        .runtime
        .branch_identity(&BranchId("ghost".to_owned()))
        .is_err());
    assert_eq!(after.catalog_count, before.catalog_count + 1);
}
