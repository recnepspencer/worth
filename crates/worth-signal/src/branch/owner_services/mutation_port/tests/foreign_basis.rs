use worth_foundational::FoundationalBranchReferenceMismatchAxis;

use crate::branch::{
    validate_signal_branch_name, SignalBranchAdvanceDenial, SignalBranchForkOperationDenial,
    SignalBranchRestoreDenial, SignalBranchSnapshotCaptureDenial,
};

use super::super::super::SignalOwnerCancellationSource;
use super::world::MutationWorld;

macro_rules! assert_basis_mismatch {
    ($result:expr, $denial:ident, $expected_axes:expr) => {
        match $result {
            Err($denial::BasisMismatch { axes }) => {
                assert_eq!(axes.as_slice(), $expected_axes)
            }
            Err(denial) => panic!("expected exact basis mismatch, got {denial:?}"),
            Ok(_) => panic!("equal-number foreign basis unexpectedly succeeded"),
        }
    };
}

macro_rules! assert_unknown_branch {
    ($result:expr, $denial:ident, $expected_branch_id:expr) => {
        match $result {
            Err($denial::UnknownBranch { branch_id }) => {
                assert_eq!(branch_id, $expected_branch_id)
            }
            Err(denial) => panic!("expected exact unknown branch, got {denial:?}"),
            Ok(_) => panic!("foreign basis without a receiving cell unexpectedly succeeded"),
        }
    };
}

#[test]
fn equal_branch_number_foreign_basis_matrix_returns_exact_basis_mismatch() {
    let world_a = MutationWorld::<()>::new();
    let world_b = MutationWorld::<()>::new();
    let captured_a = world_a
        .port
        .capture_exact(
            &world_a.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("owner A captures its real snapshot");
    let captured_b = world_b
        .port
        .capture_exact(
            &world_b.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("owner B captures its real snapshot");
    let local_basis = captured_a.captured_basis();
    let foreign_basis = captured_b.captured_basis();
    assert_eq!(
        local_basis.owner_branch_id(),
        foreign_basis.owner_branch_id()
    );
    assert_eq!(
        local_basis.observation().generation(),
        foreign_basis.observation().generation()
    );
    let expected_axes = [
        FoundationalBranchReferenceMismatchAxis::BranchIdentity,
        FoundationalBranchReferenceMismatchAxis::TargetBasis,
    ];
    let before = world_a.owner.cost_snapshot();

    assert_basis_mismatch!(
        world_a.port.fork_exact(
            validate_signal_branch_name("foreign-source-child").expect("name validates"),
            foreign_basis,
            &SignalOwnerCancellationSource::new().token(),
        ),
        SignalBranchForkOperationDenial,
        &expected_axes
    );
    assert_basis_mismatch!(
        world_a.port.advance_exact(
            foreign_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| panic!("basis mismatch must precede the transaction callback"),
        ),
        SignalBranchAdvanceDenial,
        &expected_axes
    );
    assert_basis_mismatch!(
        world_a
            .port
            .capture_exact(foreign_basis, &SignalOwnerCancellationSource::new().token()),
        SignalBranchSnapshotCaptureDenial,
        &expected_axes
    );
    assert_basis_mismatch!(
        world_a.port.restore_exact(
            foreign_basis,
            captured_a.admitted_snapshot(),
            &SignalOwnerCancellationSource::new().token(),
        ),
        SignalBranchRestoreDenial,
        &expected_axes
    );

    let after = world_a.owner.cost_snapshot();
    assert_eq!(
        after.owner_upgrade_attempts(),
        before.owner_upgrade_attempts() + 4
    );
    assert_eq!(
        after.branch_registry_lookups(),
        before.branch_registry_lookups() + 4
    );
    assert_eq!(
        after.branch_registry_reservations(),
        before.branch_registry_reservations() + 1
    );
    assert_eq!(
        after.retention_registry_contacts(),
        before.retention_registry_contacts() + 4
    );
    assert_eq!(
        after.target_cell_contacts(),
        before.target_cell_contacts() + 4
    );
    assert_eq!(after.target_cell_waits(), before.target_cell_waits());
    assert_eq!(after.canonical_movements(), before.canonical_movements());
    assert_eq!(after.fork_source_captures(), before.fork_source_captures());
    assert_eq!(
        after.fork_destination_preparations(),
        before.fork_destination_preparations()
    );
    assert_eq!(
        after.fork_destination_installations(),
        before.fork_destination_installations()
    );
    assert_eq!(
        after.branch_registry_entries_scanned(),
        before.branch_registry_entries_scanned()
    );

    let before_foreign_snapshot = world_a.owner.cost_snapshot();
    assert!(matches!(
        world_a.port.restore_exact(
            local_basis,
            captured_b.admitted_snapshot(),
            &SignalOwnerCancellationSource::new().token(),
        ),
        Err(SignalBranchRestoreDenial::ForeignSnapshotOwner { .. })
    ));
    let after_foreign_snapshot = world_a.owner.cost_snapshot();
    assert_eq!(
        after_foreign_snapshot.owner_upgrade_attempts(),
        before_foreign_snapshot.owner_upgrade_attempts() + 1
    );
    assert_eq!(
        after_foreign_snapshot.branch_registry_lookups(),
        before_foreign_snapshot.branch_registry_lookups() + 1
    );
    assert_eq!(
        after_foreign_snapshot.retention_registry_contacts(),
        before_foreign_snapshot.retention_registry_contacts() + 1
    );
    assert_eq!(
        after_foreign_snapshot.target_cell_contacts(),
        before_foreign_snapshot.target_cell_contacts()
    );
    assert_eq!(
        after_foreign_snapshot.canonical_movements(),
        before_foreign_snapshot.canonical_movements()
    );
}

#[test]
fn foreign_basis_without_receiving_cell_preserves_unknown_branch_matrix() {
    let world_a = MutationWorld::<()>::new();
    let world_b = MutationWorld::<()>::new();
    let captured_a = world_a
        .port
        .capture_exact(
            &world_a.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("owner A captures the restore fixture");
    let foreign_fork = world_b
        .port
        .fork_exact(
            validate_signal_branch_name("foreign-unknown-source").expect("name validates"),
            &world_b.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("owner B issues a branch number absent from owner A");
    let unknown_basis = foreign_fork.created_basis();
    let unknown_branch_id = unknown_basis.owner_branch_id();
    assert_ne!(unknown_branch_id, world_a.source_branch.id);
    assert_ne!(unknown_branch_id, world_a.sibling_basis.owner_branch_id());
    let before = world_a.owner.cost_snapshot();

    assert_unknown_branch!(
        world_a.port.fork_exact(
            validate_signal_branch_name("unknown-source-child").expect("name validates"),
            unknown_basis,
            &SignalOwnerCancellationSource::new().token(),
        ),
        SignalBranchForkOperationDenial,
        unknown_branch_id
    );
    assert_unknown_branch!(
        world_a.port.advance_exact(
            unknown_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| panic!("unknown branch must precede the transaction callback"),
        ),
        SignalBranchAdvanceDenial,
        unknown_branch_id
    );
    assert_unknown_branch!(
        world_a
            .port
            .capture_exact(unknown_basis, &SignalOwnerCancellationSource::new().token()),
        SignalBranchSnapshotCaptureDenial,
        unknown_branch_id
    );
    assert_unknown_branch!(
        world_a.port.restore_exact(
            unknown_basis,
            captured_a.admitted_snapshot(),
            &SignalOwnerCancellationSource::new().token(),
        ),
        SignalBranchRestoreDenial,
        unknown_branch_id
    );

    let after = world_a.owner.cost_snapshot();
    assert_eq!(
        after.owner_upgrade_attempts(),
        before.owner_upgrade_attempts() + 4
    );
    assert_eq!(
        after.branch_registry_lookups(),
        before.branch_registry_lookups() + 4
    );
    assert_eq!(
        after.branch_registry_reservations(),
        before.branch_registry_reservations()
    );
    assert_eq!(
        after.retention_registry_contacts(),
        before.retention_registry_contacts()
    );
    assert_eq!(after.target_cell_contacts(), before.target_cell_contacts());
    assert_eq!(after.target_cell_waits(), before.target_cell_waits());
    assert_eq!(after.canonical_movements(), before.canonical_movements());
    assert_eq!(after.fork_source_captures(), before.fork_source_captures());
    assert_eq!(
        after.fork_destination_preparations(),
        before.fork_destination_preparations()
    );
    assert_eq!(
        after.fork_destination_installations(),
        before.fork_destination_installations()
    );
    assert_eq!(
        after.branch_registry_entries_scanned(),
        before.branch_registry_entries_scanned()
    );
}
