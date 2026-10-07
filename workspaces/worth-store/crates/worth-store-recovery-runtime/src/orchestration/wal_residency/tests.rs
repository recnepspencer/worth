//! Real pool ownership of vector slots; inline integers assert no WAL authority.

use super::*;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecoveryFilesystemMedia, FilesystemAccessPosture, FilesystemMediaAdmission,
    PhysicalOperationAllocationScope as Scope, PhysicalRecoveryCoordinationCapacity,
    PhysicalRecoveryFreshnessPort, PhysicalResidencyDimension as Dimension,
    PhysicalRuntimeAdmission, PhysicalStore, QualifiedRecoveryFilesystemMedia,
};

const ORIGINAL: u64 = 16 << 10;

#[test]
fn growth_requires_old_and_new_capacity_and_preserves_prefix_on_native_denial() {
    let (_directory, media, coordination) = world();
    let observer = coordination.certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        0
    );
    let mut roster = NativeWalRoster::<u64>::with_capacity(&coordination, 2).unwrap();
    roster.push_reserved(17);
    let spare = observer.snapshot().for_dimension(dimension);
    roster.reserve_one(&coordination).unwrap();
    assert_eq!(observer.snapshot().for_dimension(dimension), spare);
    roster.push_reserved(29);
    let old_capacity = roster.capacity();
    let old_bytes = (old_capacity * std::mem::size_of::<u64>()) as u64;
    assert_eq!(old_capacity, 2);
    assert_eq!(roster.charged_bytes(), old_bytes);
    let pointer = roster.as_slice().as_ptr();
    let prospective_bytes = (4 * std::mem::size_of::<u64>()) as u64;
    let held = coordination
        .certification_begin_recovery_allocation(
            NonZeroU64::new(ORIGINAL - old_bytes - prospective_bytes + 1).unwrap(),
        )
        .unwrap();
    let before = observer.snapshot().for_dimension(dimension);
    assert_eq!(
        roster.reserve_one(&coordination).unwrap_err(),
        Denial::Backing {
            requested: old_bytes + prospective_bytes,
            cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                required: ORIGINAL + 1,
                admitted: ORIGINAL,
            },
        }
    );
    assert_eq!(roster.as_slice(), &[17, 29]);
    assert_eq!(roster.as_slice().as_ptr(), pointer);
    assert_eq!(roster.capacity(), old_capacity);
    assert_eq!(roster.charged_bytes(), old_bytes);
    let denied = observer.snapshot().for_dimension(dimension);
    assert_eq!(denied.admissions(), before.admissions());
    assert_eq!(denied.admitted_units(), before.admitted_units());
    assert_eq!(denied.released_units(), before.released_units());
    assert_eq!(denied.active_units(), before.active_units());
    assert_eq!(denied.denials(), before.denials() + 1);
    drop(held);

    let retry_before = observer.snapshot().for_dimension(dimension);
    roster.reserve_one(&coordination).unwrap();
    assert_eq!(roster.capacity(), 4);
    assert_eq!(roster.as_slice(), &[17, 29]);
    let retained = (roster.capacity() * std::mem::size_of::<u64>()) as u64;
    assert_eq!(roster.charged_bytes(), retained);
    let retry = observer.snapshot().for_dimension(dimension);
    assert_eq!(retry.admissions(), retry_before.admissions() + 1);
    assert_eq!(
        retry.admitted_units() - retry_before.admitted_units(),
        prospective_bytes
    );
    assert_eq!(
        retry.released_units() - retry_before.released_units(),
        old_bytes
    );
    assert_eq!(retry.active_units(), retained);
    roster.push_reserved(41);
    roster.as_mut_slice()[0] = 53;
    assert_eq!(roster.as_slice(), &[53, 29, 41]);
    assert_eq!(roster.owned_heap_bytes(), Some(retained));
    drop(coordination);
    drop(media);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        retained
    );
    drop(roster);
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}

#[test]
fn empty_roster_has_no_backing_and_shared_storage_is_prefunded_with_first_slot() {
    let (_directory, media, coordination) = world();
    let observer = coordination.certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let before = observer.snapshot().for_dimension(dimension);
    let mut roster = NativeWalRoster::<u64>::empty(73);
    assert!(roster.is_empty());
    assert_eq!(roster.charged_bytes(), 0);
    assert_eq!(roster.owned_heap_bytes(), Some(0));
    assert_eq!(observer.snapshot().for_dimension(dimension), before);
    roster.reserve_one(&coordination).unwrap();
    roster.push_reserved(7);
    let expected = 73 + (roster.capacity() * std::mem::size_of::<u64>()) as u64;
    assert_eq!(roster.charged_bytes(), expected);
    assert_eq!(roster.owned_heap_bytes(), Some(expected));
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        expected
    );
    drop(roster);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        0
    );
    drop(coordination);
    drop(media);
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}

fn world() -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("actual Store namespace must initialize");
    };
    media.close();
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(&root).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let capacity = PhysicalRecoveryCoordinationCapacity::admit(2, 4096, 2, 4096)
        .unwrap()
        .with_recovery_allocation_bytes(ORIGINAL)
        .unwrap();
    let coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            capacity,
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    (directory, media, coordination)
}
