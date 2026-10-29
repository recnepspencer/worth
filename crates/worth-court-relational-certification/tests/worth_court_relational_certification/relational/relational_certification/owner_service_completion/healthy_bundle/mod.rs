pub(crate) use {std::collections::BTreeSet};

pub(crate) use {crate::relational_certification::ordinary_context::owner_service_completion::world::commit_through_services, crate::relational_certification::ordinary_context::owner_service_completion::world::empty_runtime, crate::relational_certification::ordinary_context::owner_service_completion::world::fork_through_services};
pub(crate) use {worth_relational::facade::branch::RelationalBranchBasisDenial, worth_relational::facade::branch::RelationalBranchDeletionOutcome, worth_relational::facade::branch::RelationalOwnerLifecycleObservation};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::mvcc::RelationalTransactionIntent};

#[test]
fn healthy_bundle_executes_every_promised_method_family() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = empty_runtime();
    let services = runtime.owner_component_services();
    assert_eq!(
        services.lifecycle_port().owner_lifecycle_observation(),
        worth_relational::facade::branch::RelationalOwnerLifecycleObservation::Open
    );

    let committed = commit_through_services(execution, &runtime, &services, "six-port-health");
    assert_eq!(committed.commit.branch_id, BranchId("main".to_owned()));
    let archived_identity = fork_through_services(execution, &services, "archived")
        .target_identity()
        .clone();
    let deleted_identity = fork_through_services(execution, &services, "deleted")
        .target_identity()
        .clone();

    let basis = services.basis_port();
    let (descriptor, admitted) = basis
        .observe_branch(&archived_identity)
        .expect("basis service observes the fork");
    assert_eq!(
        basis
            .observe_branch_with_control(
                &archived_identity,
                &worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted(),
            )
            .expect("controlled observation reaches the same canonical method")
            .0,
        descriptor
    );
    assert_eq!(
        basis
            .admit_branch_basis(&archived_identity)
            .expect("identity admission succeeds")
            .descriptor(),
        &descriptor
    );
    assert_eq!(
        basis
            .readmit_branch_basis(&descriptor)
            .expect("descriptor readmission consults live owner truth")
            .descriptor(),
        &descriptor
    );
    let lease = basis
        .retain_component_basis(&admitted)
        .expect("owner retains the admitted exact basis");
    assert_eq!(
        basis
            .readmit_retained_branch_basis(&descriptor, &lease)
            .expect("live exact lease readmits its basis")
            .descriptor(),
        &descriptor
    );
    basis
        .release_component_basis(lease)
        .expect("the same owner explicitly releases the lease");

    services
        .lifecycle_port()
        .archive_branch(execution, &archived_identity)
        .expect("lifecycle service archives the exact branch");
    assert!(matches!(
        runtime.observe_branch(&archived_identity),
        Err(worth_relational::facade::branch::RelationalBranchBasisDenial::ArchivedBranch(branch))
            if branch == BranchId("archived".to_owned())
    ));
    assert!(matches!(
        services
            .lifecycle_port()
            .delete_branch(execution, &deleted_identity)
            .expect("lifecycle service deletes an unretained branch"),
        worth_relational::facade::branch::RelationalBranchDeletionOutcome::Deleted(deleted)
            if deleted.identity() == &deleted_identity
    ));
    assert!(runtime
        .branch_identity(&BranchId("deleted".to_owned()))
        .is_err());
}
