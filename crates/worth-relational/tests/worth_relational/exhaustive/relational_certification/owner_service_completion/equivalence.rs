#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::owner_service_completion::equivalence::*;

#[test]
fn basis_and_lifecycle_routes_share_exact_artifacts_on_one_owner() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use super::super::world::supply_chain::{certified_supply_chain_world, SupplyChainScale};

    let (mut world, _baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let services = world.runtime.owner_component_services();
    let main = world.runtime.main_branch_identity();
    let (direct_descriptor, direct_basis) = world
        .runtime
        .observe_branch(&main)
        .expect("the direct route issues the canonical main basis");
    let port_readmitted = services
        .basis_port()
        .readmit_branch_basis(&direct_descriptor)
        .expect("the port readmits the direct route's exact descriptor");
    assert_eq!(port_readmitted.descriptor(), &direct_descriptor);
    let lease = services
        .basis_port()
        .retain_component_basis(&direct_basis)
        .expect("the port retains the direct route's exact basis");
    assert_eq!(
        world
            .runtime
            .readmit_retained_branch_basis(&direct_descriptor, &lease)
            .expect("the direct route readmits the port-issued lease")
            .descriptor(),
        &direct_descriptor
    );
    assert_eq!(
        world
            .runtime
            .release_component_basis(lease)
            .expect("the direct route releases the port-issued lease")
            .outcome(),
        RelationalBranchRetentionTerminalOutcome::Released
    );

    let (_, port_source) = services
        .fork_port()
        .observe_fork_source(main.branch_id())
        .expect("the port observes the non-empty canonical source");
    let port_target = services
        .fork_port()
        .fork_branch(execution, BranchId("port-catalog".to_owned()), port_source)
        .expect("the port installs its target in the canonical catalog")
        .target_identity()
        .clone();
    world
        .runtime
        .observe_branch(&port_target)
        .expect("the direct route observes the port-installed target");

    let (_, direct_source) = world
        .runtime
        .observe_fork_source(main.branch_id())
        .expect("the direct route observes the same canonical source");
    let direct_target = world
        .runtime
        .fork_branch(
            execution,
            BranchId("direct-catalog".to_owned()),
            direct_source,
        )
        .expect("the direct route installs its target in the canonical catalog")
        .target_identity()
        .clone();
    services
        .basis_port()
        .observe_branch(&direct_target)
        .expect("the port observes the direct-installed target");

    assert!(matches!(
        world.runtime.delete_branch(execution, &port_target),
        Ok(RelationalBranchDeletionOutcome::Deleted(ref deleted))
            if deleted.identity() == &port_target
    ));
    assert!(matches!(
        services.basis_port().observe_branch(&port_target),
        Err(RelationalBranchBasisDenial::UnknownBranch(ref branch))
            if branch == port_target.branch_id()
    ));
    assert!(matches!(
        services.lifecycle_port().delete_branch(execution, &direct_target),
        Ok(RelationalBranchDeletionOutcome::Deleted(ref deleted))
            if deleted.identity() == &direct_target
    ));
    assert!(matches!(
        world.runtime.observe_branch(&direct_target),
        Err(RelationalBranchBasisDenial::UnknownBranch(ref branch))
            if branch == direct_target.branch_id()
    ));
}
