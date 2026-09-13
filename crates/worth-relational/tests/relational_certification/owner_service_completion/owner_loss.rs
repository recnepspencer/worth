use super::world::{commit_through_services, empty_runtime, fork_through_services};
use worth_relational::facade::branch::{
    RelationalBranchArchiveDenial, RelationalBranchBasisDenial, RelationalBranchDeleteDenial,
    RelationalOwnerLifecycleObservation,
};
use worth_relational::facade::mvcc::RelationalOperationControl;

#[test]
fn surviving_bundle_is_weak_and_reports_exact_owner_unavailable_postures() {
    let runtime = empty_runtime();
    let services = runtime.owner_component_services();
    commit_through_services(&runtime, &services, "owner-loss-world");
    let identity = fork_through_services(&services, "owner-loss")
        .target_identity()
        .clone();
    let (descriptor, basis) = services.basis_port().observe_branch(&identity).unwrap();
    let lease = services
        .basis_port()
        .retain_component_basis(&basis)
        .unwrap();
    drop(runtime);

    assert_eq!(
        services.lifecycle_port().owner_lifecycle_observation(),
        RelationalOwnerLifecycleObservation::Closed
    );
    assert!(matches!(
        services.basis_port().observe_branch(&identity),
        Err(RelationalBranchBasisDenial::OwnerUnavailable)
    ));
    assert!(matches!(
        services
            .basis_port()
            .observe_branch_with_control(&identity, &RelationalOperationControl::uninterrupted(),),
        Err(RelationalBranchBasisDenial::OwnerUnavailable)
    ));
    assert!(matches!(
        services.basis_port().admit_branch_basis(&identity),
        Err(RelationalBranchBasisDenial::OwnerUnavailable)
    ));
    assert!(matches!(
        services.basis_port().readmit_branch_basis(&descriptor),
        Err(RelationalBranchBasisDenial::OwnerUnavailable)
    ));
    assert!(matches!(
        services
            .basis_port()
            .readmit_retained_branch_basis(&descriptor, &lease),
        Err(RelationalBranchBasisDenial::OwnerUnavailable)
    ));
    assert!(matches!(
        services.basis_port().retain_component_basis(&basis),
        Err(RelationalBranchBasisDenial::OwnerUnavailable)
    ));
    assert!(matches!(
        services.lifecycle_port().archive_branch(&identity),
        Err(RelationalBranchArchiveDenial::OwnerUnavailable)
    ));
    assert!(matches!(
        services.lifecycle_port().delete_branch(&identity),
        Err(RelationalBranchDeleteDenial::OwnerUnavailable)
    ));
    let release = services
        .basis_port()
        .release_component_basis(lease)
        .expect_err("a lost owner cannot freshly admit the release route");
    assert_eq!(
        release.denial(),
        &RelationalBranchBasisDenial::OwnerUnavailable
    );
    drop(release.into_lease());
    drop(basis);
}
