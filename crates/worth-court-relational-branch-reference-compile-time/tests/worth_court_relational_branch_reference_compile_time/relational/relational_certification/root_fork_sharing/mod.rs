#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, crate::relational_certification::ordinary_context::root_fork_sharing::branch_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, crate::relational_certification::ordinary_context::root_fork_sharing::fork_from_main};

#[test]
fn fork_observation_reports_shared_root_and_distinct_cells() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, expected) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &expected);
    fork_from_main(execution, &world.runtime, "storm");
    fork_from_main(execution, &world.runtime, "maintenance");

    let identities = [
        world.runtime.main_branch_identity(),
        branch_identity(&world.runtime, "storm"),
        branch_identity(&world.runtime, "maintenance"),
    ];
    let observation = world.runtime.observe_branch_sharing(&identities).unwrap();
    assert_eq!(
        observation.inspection_version(),
        worth_relational::facade::inspection::RELATIONAL_SHARING_INSPECTION_VERSION
    );
    assert_eq!(
        observation.byte_metric_scope(),
        worth_relational::facade::inspection::RelationalSharingByteMetricScope::CompleteAuthoritativeOwnerAllocations
    );
    assert_eq!(observation.branch_count(), 3);
    assert_eq!(observation.unique_root_count(), 1);
    assert_eq!(observation.unique_canonical_commit_artifacts(), 1);
    assert_eq!(observation.copied_truth_bytes(), 0);
    assert_eq!(observation.copied_commit_envelopes(), 0);
    assert_eq!(observation.shared_root_acquisitions(), 2);
    assert_eq!(observation.fork_materialized_entity_count(), 0);
    assert_eq!(observation.fork_materialized_relation_count(), 0);
    assert_eq!(observation.coordination_waits(), 0);
    assert_eq!(
        observation.logical_branch_partition_payload_bytes(),
        observation.unique_physical_partition_payload_bytes() * observation.branch_count()
    );
    assert_eq!(observation.coordination_cells().len(), 3);
    assert_eq!(
        observation
            .coordination_cells()
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
}
