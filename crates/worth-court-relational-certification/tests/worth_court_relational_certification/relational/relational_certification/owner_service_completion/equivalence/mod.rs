#[allow(unused_imports)] use crate::relational_certification::ordinary_context::owner_service_completion::equivalence::*;
pub(crate) use {crate::relational_certification::ordinary_context::owner_service_completion::world::commit_direct, crate::relational_certification::ordinary_context::owner_service_completion::world::commit_through_services, crate::relational_certification::ordinary_context::owner_service_completion::world::empty_runtime, crate::relational_certification::ordinary_context::owner_service_completion::world::fork_direct, crate::relational_certification::ordinary_context::owner_service_completion::world::fork_through_services};
pub(crate) use {worth_relational::facade::branch::RelationalBranchBasisDenial, worth_relational::facade::branch::RelationalBranchBasisDescriptor, worth_relational::facade::branch::RelationalBranchDeletionOutcome, worth_relational::facade::branch::RelationalBranchLifecyclePosture, worth_relational::facade::branch::RelationalBranchRetentionTerminalOutcome};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::mvcc::RelationalOperationControl};

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
                &worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted(),
            )
            .expect("direct controlled observation succeeds")
            .0,
        &services
            .basis_port()
            .observe_branch_with_control(
                &ported_identity,
                &worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted(),
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
        worth_relational::facade::branch::RelationalBranchRetentionTerminalOutcome::Released
    );
    assert_eq!(
        services
            .basis_port()
            .release_component_basis(ported_lease)
            .expect("ported lease release succeeds")
            .outcome(),
        worth_relational::facade::branch::RelationalBranchRetentionTerminalOutcome::Released
    );
}

#[test]
fn direct_and_service_routes_have_equivalent_semantics_on_separate_worlds() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut direct = empty_runtime();
    let ported = empty_runtime();
    let services = ported.owner_component_services();

    let direct_commit = commit_direct(execution, &direct, "equivalent-movement");
    let port_commit = commit_through_services(execution, &ported, &services, "equivalent-movement");
    assert_eq!(direct_commit.commit.branch_id, port_commit.commit.branch_id);
    assert_eq!(
        direct_commit.commit.version_id,
        port_commit.commit.version_id
    );
    assert_eq!(direct_commit.patch().len(), port_commit.patch().len());

    let direct_identity = fork_direct(execution, &direct, "equivalent")
        .target_identity()
        .clone();
    let port_identity = fork_through_services(execution, &services, "equivalent")
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

    let direct_archived = direct.archive_branch(execution, &direct_identity).unwrap();
    let port_archived = services
        .lifecycle_port()
        .archive_branch(execution, &port_identity)
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
        worth_relational::facade::branch::RelationalBranchLifecyclePosture::Archived
    );
    assert_eq!(
        ported
            .branch_reference_state(port_identity.branch_id())
            .unwrap()
            .lifecycle_posture(),
        worth_relational::facade::branch::RelationalBranchLifecyclePosture::Archived
    );

    let direct_deleted = fork_direct(execution, &direct, "equivalent-deleted")
        .target_identity()
        .clone();
    let ported_deleted = fork_through_services(execution, &services, "equivalent-deleted")
        .target_identity()
        .clone();
    assert!(matches!(
        direct.delete_branch(execution, &direct_deleted).unwrap(),
        worth_relational::facade::branch::RelationalBranchDeletionOutcome::Deleted(ref deleted)
            if deleted.identity().branch_id() == direct_deleted.branch_id()
    ));
    assert!(matches!(
        services
            .lifecycle_port()
            .delete_branch(execution, &ported_deleted)
            .unwrap(),
        worth_relational::facade::branch::RelationalBranchDeletionOutcome::Deleted(ref deleted)
            if deleted.identity().branch_id() == ported_deleted.branch_id()
    ));
}

#[test]
fn movement_through_either_route_is_immediately_visible_through_the_other() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = empty_runtime();
    let services = runtime.owner_component_services();
    commit_through_services(execution, &runtime, &services, "shared-state-visibility");
    let port_archived = fork_through_services(execution, &services, "port-archived")
        .target_identity()
        .clone();
    let direct_archived = fork_through_services(execution, &services, "direct-archived")
        .target_identity()
        .clone();

    services
        .lifecycle_port()
        .archive_branch(execution, &port_archived)
        .unwrap();
    assert!(matches!(
        runtime.observe_branch(&port_archived),
        Err(worth_relational::facade::branch::RelationalBranchBasisDenial::ArchivedBranch(_))
    ));

    runtime.archive_branch(execution, &direct_archived).unwrap();
    assert!(matches!(
        services.basis_port().observe_branch(&direct_archived),
        Err(worth_relational::facade::branch::RelationalBranchBasisDenial::ArchivedBranch(_))
    ));
}
