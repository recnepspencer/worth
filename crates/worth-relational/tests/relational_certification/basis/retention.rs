use std::collections::BTreeSet;

use super::world::supply_chain::{
    certified_supply_chain_world, commit_branch_batch, fork_supply_chain_branch_from_main,
    lower_supply_chain_production_delta, DeltaId, SupplyChainScale,
};
use worth_relational::facade::branch::RelationalBranchBasisDenial;
use worth_relational::facade::history::BranchId;

#[test]
fn external_component_pin_is_the_last_readmission_obligation() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let runtime = world.runtime;
    let program = world.program;
    let handles = world.handles;
    let branch_id = BranchId("storm".to_owned());
    fork_supply_chain_branch_from_main(execution, &runtime, branch_id.clone());
    let identity = runtime.branch_identity(&branch_id).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let descriptor = basis.descriptor().clone();
    let external = runtime.retain_component_basis(&basis).unwrap();

    let batch = lower_supply_chain_production_delta(
        execution,
        &runtime,
        &program,
        &handles,
        &branch_id,
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .unwrap();
    commit_branch_batch(execution, &runtime, branch_id, batch);
    drop(basis);

    let retained = runtime
        .readmit_branch_basis(&descriptor)
        .expect("the explicit external obligation retains the old basis");
    assert_eq!(retained.descriptor(), &descriptor);
    drop(retained);

    let receipt = runtime.release_component_basis(external).unwrap();
    assert_eq!(receipt.descriptor(), &descriptor);
    assert!(matches!(
        runtime.readmit_branch_basis(&descriptor),
        Err(RelationalBranchBasisDenial::UnavailableRetainedTarget)
    ));
}

#[test]
fn foreign_release_denial_preserves_the_owner_obligation() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let runtime = world.runtime;
    let program = world.program;
    let handles = world.handles;
    let branch_id = BranchId("storm".to_owned());
    fork_supply_chain_branch_from_main(execution, &runtime, branch_id.clone());
    let identity = runtime.branch_identity(&branch_id).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let descriptor = basis.descriptor().clone();
    let external = runtime.retain_component_basis(&basis).unwrap();

    let batch = lower_supply_chain_production_delta(
        execution,
        &runtime,
        &program,
        &handles,
        &branch_id,
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .unwrap();
    commit_branch_batch(execution, &runtime, branch_id, batch);
    drop(basis);

    let foreign = certified_supply_chain_world(execution, SupplyChainScale::court())
        .0
        .runtime;
    let owner_before_denial = runtime.branch_basis_cost_counters();
    let foreign_before_denial = foreign.branch_basis_cost_counters();
    let denied = foreign
        .release_component_basis(external)
        .expect_err("a foreign runtime cannot consume the owner's obligation");
    assert!(matches!(
        denied.denial(),
        RelationalBranchBasisDenial::ForeignRuntime { .. }
    ));
    assert_eq!(
        runtime.branch_basis_cost_counters(),
        owner_before_denial,
        "foreign denial must not terminate the owner obligation"
    );
    assert_eq!(
        foreign.branch_basis_cost_counters(),
        foreign_before_denial,
        "foreign denial must not record owner lifecycle work"
    );

    let external = denied.into_lease();
    let retained = runtime
        .readmit_branch_basis(&descriptor)
        .expect("the denied lease still retains the old owner basis");
    drop(retained);
    runtime.release_component_basis(external).unwrap();
    assert!(matches!(
        runtime.readmit_branch_basis(&descriptor),
        Err(RelationalBranchBasisDenial::UnavailableRetainedTarget)
    ));
}

#[test]
fn dropping_external_retention_uses_the_same_terminal_accounting_path() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let runtime = world.runtime;
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let baseline = runtime.branch_basis_cost_counters();

    let lease = runtime.retain_component_basis(&basis).unwrap();
    assert_eq!(
        lease.retention_reason(),
        worth_relational::facade::branch::RelationalBasisRetentionReason::ExternalComponentBasis
    );
    drop(lease);

    let observed = runtime.branch_basis_cost_counters();
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
        baseline.external_retention_drop_releases + 1
    );
}
