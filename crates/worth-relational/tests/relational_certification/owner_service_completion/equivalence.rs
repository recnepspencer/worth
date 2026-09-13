use super::world::{
    commit_direct, commit_through_services, empty_runtime, fork_direct, fork_through_services,
};
use worth_relational::facade::branch::{
    RelationalBranchBasisDenial, RelationalBranchBasisDescriptor, RelationalBranchDeletionOutcome,
    RelationalBranchLifecyclePosture, RelationalBranchRetentionTerminalOutcome,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::RelationalOperationControl;

#[test]
fn basis_port_matches_every_direct_basis_operation_on_separate_worlds() {
    let direct = empty_runtime();
    let ported = empty_runtime();
    let services = ported.owner_component_services();
    let direct_identity = direct.main_branch_identity();
    let ported_identity = ported.main_branch_identity();

    let (direct_descriptor, direct_observed) = direct
        .observe_branch(&direct_identity)
        .expect("the direct owner observes its exact basis");
    let (ported_descriptor, ported_observed) = services
        .basis_port()
        .observe_branch(&ported_identity)
        .expect("the basis port observes its exact basis");
    assert_basis_semantics_equal(&direct_descriptor, &ported_descriptor);
    assert_basis_semantics_equal(
        &direct
            .observe_branch_with_control(
                &direct_identity,
                &RelationalOperationControl::uninterrupted(),
            )
            .expect("direct controlled observation succeeds")
            .0,
        &services
            .basis_port()
            .observe_branch_with_control(
                &ported_identity,
                &RelationalOperationControl::uninterrupted(),
            )
            .expect("ported controlled observation succeeds")
            .0,
    );
    assert_basis_semantics_equal(
        direct
            .admit_branch_basis(&direct_identity)
            .expect("direct identity admission succeeds")
            .descriptor(),
        services
            .basis_port()
            .admit_branch_basis(&ported_identity)
            .expect("ported identity admission succeeds")
            .descriptor(),
    );
    assert_basis_semantics_equal(
        direct
            .readmit_branch_basis(&direct_descriptor)
            .expect("direct descriptor readmission succeeds")
            .descriptor(),
        services
            .basis_port()
            .readmit_branch_basis(&ported_descriptor)
            .expect("ported descriptor readmission succeeds")
            .descriptor(),
    );

    let direct_lease = direct
        .retain_component_basis(&direct_observed)
        .expect("direct retention succeeds");
    let ported_lease = services
        .basis_port()
        .retain_component_basis(&ported_observed)
        .expect("ported retention succeeds");
    assert_basis_semantics_equal(
        direct
            .readmit_retained_branch_basis(&direct_descriptor, &direct_lease)
            .expect("direct retained readmission succeeds")
            .descriptor(),
        services
            .basis_port()
            .readmit_retained_branch_basis(&ported_descriptor, &ported_lease)
            .expect("ported retained readmission succeeds")
            .descriptor(),
    );
    assert_eq!(
        direct
            .release_component_basis(direct_lease)
            .expect("direct lease release succeeds")
            .outcome(),
        RelationalBranchRetentionTerminalOutcome::Released
    );
    assert_eq!(
        services
            .basis_port()
            .release_component_basis(ported_lease)
            .expect("ported lease release succeeds")
            .outcome(),
        RelationalBranchRetentionTerminalOutcome::Released
    );
}

#[test]
fn direct_and_service_routes_have_equivalent_semantics_on_separate_worlds() {
    let mut direct = empty_runtime();
    let ported = empty_runtime();
    let services = ported.owner_component_services();

    let direct_commit = commit_direct(&direct, "equivalent-movement");
    let port_commit = commit_through_services(&ported, &services, "equivalent-movement");
    assert_eq!(direct_commit.commit.branch_id, port_commit.commit.branch_id);
    assert_eq!(
        direct_commit.commit.version_id,
        port_commit.commit.version_id
    );
    assert_eq!(direct_commit.patch().len(), port_commit.patch().len());

    let direct_identity = fork_direct(&direct, "equivalent").target_identity().clone();
    let port_identity = fork_through_services(&services, "equivalent")
        .target_identity()
        .clone();
    let (direct_descriptor, _) = direct.observe_branch(&direct_identity).unwrap();
    let (port_descriptor, _) = services
        .basis_port()
        .observe_branch(&port_identity)
        .unwrap();
    assert_eq!(direct_descriptor.branch_id(), port_descriptor.branch_id());
    assert_eq!(
        direct_descriptor.reference().generation(),
        port_descriptor.reference().generation()
    );
    assert_eq!(
        direct_descriptor
            .reference()
            .target()
            .as_basis()
            .expect("committed direct target")
            .roots(),
        port_descriptor
            .reference()
            .target()
            .as_basis()
            .expect("committed service target")
            .roots()
    );
    assert_eq!(
        direct_descriptor.truth_version(),
        port_descriptor.truth_version()
    );
    assert_eq!(direct_descriptor.posture(), port_descriptor.posture());

    let direct_archived = direct.archive_branch(&direct_identity).unwrap();
    let port_archived = services
        .lifecycle_port()
        .archive_branch(&port_identity)
        .unwrap();
    assert_eq!(
        direct_archived.observation().generation(),
        port_archived.observation().generation()
    );
    assert_eq!(
        direct
            .branch_reference_state(direct_identity.branch_id())
            .unwrap()
            .lifecycle_posture(),
        RelationalBranchLifecyclePosture::Archived
    );
    assert_eq!(
        ported
            .branch_reference_state(port_identity.branch_id())
            .unwrap()
            .lifecycle_posture(),
        RelationalBranchLifecyclePosture::Archived
    );

    let direct_deleted = fork_direct(&direct, "equivalent-deleted")
        .target_identity()
        .clone();
    let ported_deleted = fork_through_services(&services, "equivalent-deleted")
        .target_identity()
        .clone();
    assert!(matches!(
        direct.delete_branch(&direct_deleted).unwrap(),
        RelationalBranchDeletionOutcome::Deleted(ref deleted)
            if deleted.identity().branch_id() == direct_deleted.branch_id()
    ));
    assert!(matches!(
        services
            .lifecycle_port()
            .delete_branch(&ported_deleted)
            .unwrap(),
        RelationalBranchDeletionOutcome::Deleted(ref deleted)
            if deleted.identity().branch_id() == ported_deleted.branch_id()
    ));
}

#[test]
fn movement_through_either_route_is_immediately_visible_through_the_other() {
    let mut runtime = empty_runtime();
    let services = runtime.owner_component_services();
    commit_through_services(&runtime, &services, "shared-state-visibility");
    let port_archived = fork_through_services(&services, "port-archived")
        .target_identity()
        .clone();
    let direct_archived = fork_through_services(&services, "direct-archived")
        .target_identity()
        .clone();

    services
        .lifecycle_port()
        .archive_branch(&port_archived)
        .unwrap();
    assert!(matches!(
        runtime.observe_branch(&port_archived),
        Err(RelationalBranchBasisDenial::ArchivedBranch(_))
    ));

    runtime.archive_branch(&direct_archived).unwrap();
    assert!(matches!(
        services.basis_port().observe_branch(&direct_archived),
        Err(RelationalBranchBasisDenial::ArchivedBranch(_))
    ));
}

#[test]
fn basis_and_lifecycle_routes_share_exact_artifacts_on_one_owner() {
    use super::super::world::supply_chain::{certified_supply_chain_world, SupplyChainScale};

    let (mut world, _baseline) = certified_supply_chain_world(SupplyChainScale::court());
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
        .fork_branch(BranchId("port-catalog".to_owned()), port_source)
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
        .fork_branch(BranchId("direct-catalog".to_owned()), direct_source)
        .expect("the direct route installs its target in the canonical catalog")
        .target_identity()
        .clone();
    services
        .basis_port()
        .observe_branch(&direct_target)
        .expect("the port observes the direct-installed target");

    assert!(matches!(
        world.runtime.delete_branch(&port_target),
        Ok(RelationalBranchDeletionOutcome::Deleted(ref deleted))
            if deleted.identity() == &port_target
    ));
    assert!(matches!(
        services.basis_port().observe_branch(&port_target),
        Err(RelationalBranchBasisDenial::UnknownBranch(ref branch))
            if branch == port_target.branch_id()
    ));
    assert!(matches!(
        services.lifecycle_port().delete_branch(&direct_target),
        Ok(RelationalBranchDeletionOutcome::Deleted(ref deleted))
            if deleted.identity() == &direct_target
    ));
    assert!(matches!(
        world.runtime.observe_branch(&direct_target),
        Err(RelationalBranchBasisDenial::UnknownBranch(ref branch))
            if branch == direct_target.branch_id()
    ));
}

fn assert_basis_semantics_equal(
    direct: &RelationalBranchBasisDescriptor,
    ported: &RelationalBranchBasisDescriptor,
) {
    assert_eq!(direct.branch_id(), ported.branch_id());
    assert_eq!(
        direct.reference().generation(),
        ported.reference().generation()
    );
    assert_eq!(direct.truth_version(), ported.truth_version());
    assert_eq!(direct.schema_commitment(), ported.schema_commitment());
    assert_eq!(
        direct.visibility_commitment(),
        ported.visibility_commitment()
    );
    assert_eq!(direct.posture(), ported.posture());
}
