use super::world::supply_chain::{certified_supply_chain_world, SupplyChainScale};
use worth_relational::facade::branch::RelationalBranchBasisDenial;

#[test]
fn basis_and_external_retention_work_is_counted_exactly() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let runtime = world.runtime;
    let baseline = runtime.branch_basis_cost_counters();
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    let after_observation = runtime.branch_basis_cost_counters();
    assert_eq!(
        after_observation.basis_observations,
        baseline.basis_observations + 1
    );
    assert_eq!(
        after_observation.retained_basis_registry_entries, baseline.retained_basis_registry_entries,
        "re-observing one exact basis reuses its canonical live registry entry"
    );

    let readmitted = runtime.readmit_branch_basis(&descriptor).unwrap();
    let foreign = certified_supply_chain_world(execution, SupplyChainScale::court())
        .0
        .runtime;
    assert!(matches!(
        foreign.readmit_branch_basis(&descriptor),
        Err(RelationalBranchBasisDenial::ForeignRuntime { .. })
    ));
    let lease = runtime.retain_component_basis(&basis).unwrap();
    runtime.release_component_basis(lease).unwrap();
    drop(readmitted);

    let observed = runtime.branch_basis_cost_counters();
    assert_eq!(
        observed.descriptor_resolution_attempts,
        baseline.descriptor_resolution_attempts + 1
    );
    assert_eq!(
        observed.readmission_successes,
        baseline.readmission_successes + 1
    );
    assert_eq!(observed.readmission_denials, baseline.readmission_denials);
    assert_eq!(
        observed.external_retention_acquires,
        baseline.external_retention_acquires + 1
    );
    assert_eq!(
        observed.external_retention_releases,
        baseline.external_retention_releases + 1
    );
    assert_eq!(
        observed.external_retention_drop_releases,
        baseline.external_retention_drop_releases
    );
}

#[test]
fn immutable_component_basis_future_holder_fanout_is_exact_through_1024() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for holder_count in [1_u64, 64, 1_024] {
        let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
        let runtime = world.runtime;
        let identity = runtime.main_branch_identity();
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        let root = basis.observation().selected_root_identity();
        let baseline = runtime.branch_basis_cost_counters();
        let leases = (0..holder_count)
            .map(|_| runtime.retain_component_basis(&basis).unwrap())
            .collect::<Vec<_>>();
        let retained = runtime.branch_basis_cost_counters();
        assert_eq!(basis.observation().selected_root_identity(), root);
        assert_eq!(
            retained.external_retention_acquires - baseline.external_retention_acquires,
            holder_count
        );
        assert_eq!(
            retained.retained_basis_registry_entries, baseline.retained_basis_registry_entries,
            "future holders share one immutable owner basis entry"
        );
        for lease in leases {
            runtime.release_component_basis(lease).unwrap();
        }
        let released = runtime.branch_basis_cost_counters();
        assert_eq!(
            released.external_retention_releases - baseline.external_retention_releases,
            holder_count
        );
    }
}
