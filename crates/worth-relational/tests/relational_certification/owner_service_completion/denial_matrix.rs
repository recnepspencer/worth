use super::world::{commit_direct, committed_fork, empty_runtime};
use worth_relational::facade::branch::{
    RelationalBranchArchiveDenial, RelationalBranchBasisDenial, RelationalBranchDeleteDenial,
    RelationalBranchDeletionOutcome, RelationalBranchRetentionTerminalOutcome,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;

#[test]
fn foreign_owner_values_keep_exact_canonical_denials() {
    let first = empty_runtime();
    let second = empty_runtime();
    let first_services = first.owner_component_services();
    let services = second.owner_component_services();
    let first_identity = first.main_branch_identity();
    let (foreign_descriptor, foreign_basis) = first_services
        .basis_port()
        .observe_branch(&first_identity)
        .expect("the foreign owner issues its own exact basis");
    let foreign_lease = first_services
        .basis_port()
        .retain_component_basis(&foreign_basis)
        .expect("the foreign owner issues its own exact lease");

    assert!(matches!(
        services.basis_port().observe_branch(&first_identity),
        Err(RelationalBranchBasisDenial::ForeignRuntime { .. })
    ));
    assert!(matches!(
        services.basis_port().admit_branch_basis(&first_identity),
        Err(RelationalBranchBasisDenial::ForeignRuntime { .. })
    ));
    assert!(matches!(
        services
            .basis_port()
            .readmit_branch_basis(&foreign_descriptor),
        Err(RelationalBranchBasisDenial::ForeignRuntime { .. })
    ));
    assert!(matches!(
        services
            .basis_port()
            .readmit_retained_branch_basis(&foreign_descriptor, &foreign_lease),
        Err(RelationalBranchBasisDenial::ForeignRuntime { .. })
    ));
    assert!(matches!(
        services.basis_port().retain_component_basis(&foreign_basis),
        Err(RelationalBranchBasisDenial::ForeignRuntime { .. })
    ));
    assert_eq!(
        services
            .basis_port()
            .release_component_basis(foreign_lease)
            .expect_err("a foreign owner cannot consume the exact lease")
            .denial(),
        &RelationalBranchBasisDenial::ForeignRuntime {
            expected_runtime_instance_id: second.main_branch_identity().runtime_instance_id(),
            actual_runtime_instance_id: first_identity.runtime_instance_id(),
        }
    );
    assert!(matches!(
        services.lifecycle_port().archive_branch(&first_identity),
        Err(RelationalBranchArchiveDenial::ForeignRuntime)
    ));
    assert!(matches!(
        services.lifecycle_port().delete_branch(&first_identity),
        Err(RelationalBranchDeleteDenial::ForeignRuntime)
    ));
}

#[test]
fn stale_unretained_descriptor_keeps_its_canonical_denial() {
    let second = empty_runtime();
    let services = second.owner_component_services();
    let identity = second.main_branch_identity();
    let established = commit_direct(&second, "establish-committed-stale-source");
    second
        .snapshots()
        .release_snapshot(&established.snapshot)
        .expect("the first commit snapshot must not retain the stale root");
    drop(established);
    let (stale_descriptor, stale_basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("owner issues the descriptor before movement");
    drop(stale_basis);
    let movement = commit_direct(&second, "stale-descriptor-movement");
    second
        .snapshots()
        .release_snapshot(&movement.snapshot)
        .expect("the moving commit snapshot must not retain the old root");
    drop(movement);
    let stale_readmission = services
        .basis_port()
        .readmit_branch_basis(&stale_descriptor);
    assert!(
        matches!(
            stale_readmission,
            Err(RelationalBranchBasisDenial::UnavailableRetainedTarget)
        ),
        "movement must deny an unretained stale descriptor exactly: {stale_readmission:?}"
    );
}

#[test]
fn archived_retained_basis_survives_until_explicit_release() {
    let (mut runtime, services, archived_identity) = committed_fork("retained-archive");
    let (descriptor, basis) = services
        .basis_port()
        .observe_branch(&archived_identity)
        .unwrap();
    let lease = services
        .basis_port()
        .retain_component_basis(&basis)
        .unwrap();
    let (wrong_descriptor, wrong_basis) = services
        .basis_port()
        .observe_branch(&runtime.main_branch_identity())
        .expect("the same owner issues a distinct branch descriptor");
    assert!(matches!(
        services
            .basis_port()
            .readmit_retained_branch_basis(&wrong_descriptor, &lease),
        Err(RelationalBranchBasisDenial::WrongImmutableTarget)
    ));
    drop(wrong_basis);
    runtime.archive_branch(&archived_identity).unwrap();
    assert!(matches!(
        services.basis_port().observe_branch(&archived_identity),
        Err(RelationalBranchBasisDenial::ArchivedBranch(_))
    ));
    assert!(matches!(
        services.basis_port().admit_branch_basis(&archived_identity),
        Err(RelationalBranchBasisDenial::ArchivedBranch(_))
    ));
    assert!(matches!(
        services.basis_port().readmit_branch_basis(&descriptor),
        Err(RelationalBranchBasisDenial::ArchivedBranch(_))
    ));
    assert_eq!(
        services
            .basis_port()
            .readmit_retained_branch_basis(&descriptor, &lease)
            .expect("the exact retained immutable basis survives archive")
            .descriptor(),
        &descriptor
    );
    let receipt = services
        .basis_port()
        .release_component_basis(lease)
        .expect("explicit release consumes the live lease");
    assert_eq!(
        receipt.outcome(),
        RelationalBranchRetentionTerminalOutcome::Released
    );
}

#[test]
fn deleting_branch_reports_pending_then_deletes_after_active_operation_releases() {
    let (runtime, services, identity) = committed_fork("pending-delete");
    let basis = services
        .basis_port()
        .admit_branch_basis(&identity)
        .expect("live target admits a transaction basis");
    let deleting_descriptor = basis.descriptor().clone();
    let transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("transaction owns the active branch operation");

    let pending = services
        .lifecycle_port()
        .delete_branch(&identity)
        .expect("deletion enters its typed pending posture");
    assert!(matches!(
        pending,
        RelationalBranchDeletionOutcome::WaitingForActiveOperations(ref waiting)
            if waiting.identity() == &identity && waiting.active_operation_count() > 0
    ));
    assert!(matches!(
        services.basis_port().observe_branch(&identity),
        Err(RelationalBranchBasisDenial::DeletingBranch(_))
    ));
    assert!(matches!(
        services.basis_port().admit_branch_basis(&identity),
        Err(RelationalBranchBasisDenial::DeletingBranch(_))
    ));
    assert!(matches!(
        services
            .basis_port()
            .readmit_branch_basis(&deleting_descriptor),
        Err(RelationalBranchBasisDenial::DeletingBranch(_))
    ));
    services
        .basis_port()
        .observe_branch(&runtime.main_branch_identity())
        .expect("deleting one branch does not deny unrelated owner basis work");

    drop(transaction);
    drop(basis);
    assert!(matches!(
        services
            .lifecycle_port()
            .delete_branch(&identity)
            .expect("released operation permits exact deletion"),
        RelationalBranchDeletionOutcome::Deleted(ref deleted) if deleted.identity() == &identity
    ));
    assert!(matches!(
        services.basis_port().observe_branch(&identity),
        Err(RelationalBranchBasisDenial::UnknownBranch(ref branch))
            if branch == identity.branch_id()
    ));
    assert!(matches!(
        services.lifecycle_port().delete_branch(&identity),
        Err(RelationalBranchDeleteDenial::UnknownBranch(ref branch))
            if branch == identity.branch_id()
    ));
}
