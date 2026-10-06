use super::scales::FintechScale;
use super::scenarios::setup_world;
use super::truth_comparison::compare_exact;
use super::truth_snapshot::FintechTruthSnapshot;
use crate::facade::*;

#[test]
fn fintech_partition_locality_surfaces_changed_region_pressure_through_locality_nodes() {
    let mut world = setup_world();
    world.assert_shape(FintechScale::smoke());

    let baseline_rates = world.read_rates_partition().unwrap();
    let baseline_credit = world.read_credit_partition().unwrap();
    let baseline_detail = world.read_rates_bucket_zero().unwrap();
    let baseline_coarse = world.read_coarse_partition_book().unwrap();

    world.shock_rates_bucket_zero(7).unwrap();

    assert_eq!(
        world.node_state(world.rates_partition_node()).unwrap(),
        NodeState::Dirty
    );
    assert_eq!(
        world.node_state(world.credit_partition_node()).unwrap(),
        NodeState::Clean
    );
    assert_eq!(
        world.node_state(world.rates_bucket_zero_node()).unwrap(),
        NodeState::Dirty
    );

    let rates_after_rates_shock = world.read_rates_partition().unwrap();
    assert_ne!(rates_after_rates_shock, baseline_rates);
    let detail_after_rates_shock = world.read_rates_bucket_zero().unwrap();
    let coarse_after_rates_shock = world.read_coarse_partition_book().unwrap();
    assert_ne!(detail_after_rates_shock, baseline_detail);
    assert_ne!(coarse_after_rates_shock, baseline_coarse);

    world.shock_credit_partition(5).unwrap();

    assert_eq!(
        world.node_state(world.rates_partition_node()).unwrap(),
        NodeState::Clean
    );
    assert_eq!(
        world.node_state(world.credit_partition_node()).unwrap(),
        NodeState::Dirty
    );
    assert_eq!(
        world.node_state(world.rates_bucket_zero_node()).unwrap(),
        NodeState::Clean
    );

    let credit_after_credit_shock = world.read_credit_partition().unwrap();
    let coarse_after_credit_shock = world.read_coarse_partition_book().unwrap();

    assert_ne!(credit_after_credit_shock, baseline_credit);
    assert_ne!(coarse_after_credit_shock, coarse_after_rates_shock);
}

#[test]
fn fintech_partition_locality_checkpoint_restore_recovers_branch_local_truth_without_cross_partition_leakage(
) {
    let mut world = setup_world();
    world.assert_shape(FintechScale::smoke());

    let baseline = world.capture_active_checkpoint().unwrap();
    let analysis = world.open_branch("analysis-locality").unwrap();

    world.shock_rates_bucket_zero(9).unwrap();
    let rates_checkpoint = world.capture_active_checkpoint().unwrap();
    let rates_snapshot = (
        world.read_rates_partition().unwrap(),
        world.read_credit_partition().unwrap(),
        world.read_rates_bucket_zero().unwrap(),
    );
    let rates_truth = FintechTruthSnapshot::capture_core(&mut world).unwrap();

    world.shock_credit_partition(6).unwrap();
    let credit_snapshot = (
        world.read_rates_partition().unwrap(),
        world.read_credit_partition().unwrap(),
        world.read_rates_bucket_zero().unwrap(),
    );
    let credit_truth = FintechTruthSnapshot::capture_core(&mut world).unwrap();

    assert_ne!(rates_snapshot.1, credit_snapshot.1);
    let mismatch = compare_exact(&rates_truth, &credit_truth);
    assert!(!mismatch.is_empty());
    assert!(mismatch
        .fields
        .iter()
        .any(|field| field == "credit_partition"));

    world.restore_checkpoint(&rates_checkpoint).unwrap();
    let restored_rates = (
        world.read_rates_partition().unwrap(),
        world.read_credit_partition().unwrap(),
        world.read_rates_bucket_zero().unwrap(),
    );
    assert_eq!(restored_rates, rates_snapshot);
    let restored_truth = FintechTruthSnapshot::capture_core(&mut world).unwrap();
    assert!(compare_exact(&rates_truth, &restored_truth).is_empty());

    world.switch_branch(baseline.branch.clone()).unwrap();
    world.restore_checkpoint(&baseline).unwrap();
    let baseline_main = (
        world.read_rates_partition().unwrap(),
        world.read_credit_partition().unwrap(),
        world.read_rates_bucket_zero().unwrap(),
    );

    world.switch_branch(analysis).unwrap();
    let analysis_replay = world.replay_for_branch(world.current_branch());
    assert!(analysis_replay
        .frames
        .iter()
        .all(|frame| frame.branch_id == world.current_branch().id));
    assert_ne!(baseline_main, restored_rates);
}
