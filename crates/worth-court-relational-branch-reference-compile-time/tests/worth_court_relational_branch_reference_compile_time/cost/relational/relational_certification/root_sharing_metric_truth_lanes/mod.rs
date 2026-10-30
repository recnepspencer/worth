#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, crate::relational_certification::ordinary_context::root_sharing_metric_truth_lanes::branch_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, crate::relational_certification::ordinary_context::root_sharing_metric_truth_lanes::fork_from_main, crate::relational_certification::ordinary_context::root_sharing_metric_truth_lanes::live_byte_lanes, crate::relational_certification::ordinary_context::root_sharing_metric_truth_lanes::observe, crate::relational_certification::ordinary_context::root_sharing_metric_truth_lanes::observe_seeded_main};

#[test]
fn selection_lane_is_independent_of_live_owner_storage() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let court = observe_seeded_main(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let standard = observe_seeded_main(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::standard());

    assert_eq!(court.branch_count(), 1);
    assert_eq!(standard.branch_count(), 1);

    assert!(
        standard.unique_physical_authoritative_bytes()
            > court.unique_physical_authoritative_bytes(),
        "the two worlds must hold visibly different authoritative storage"
    );
    assert_eq!(
        court.branch_metadata_bytes(),
        standard.branch_metadata_bytes(),
        "branch metadata is a selection-lane metric: equal selections report \
         equal bytes however much authoritative storage the roots hold"
    );

    for world in [&court, &standard] {
        for (lane, live_bytes) in live_byte_lanes(world) {
            assert_ne!(
                world.branch_metadata_bytes(),
                live_bytes,
                "branch metadata must not be sourced from the live {lane} lane"
            );
        }
    }
}
