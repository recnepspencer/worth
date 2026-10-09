//! Native storage and public-clone lifetime, not a synthetic WAL admission claim.

use super::*;
use std::{alloc::Layout, num::NonZeroU64, sync::atomic::AtomicUsize};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecoveryFilesystemMedia, FilesystemAccessPosture, FilesystemMediaAdmission,
    PhysicalOperationAllocationScope as Scope, PhysicalRecoveryCoordination,
    PhysicalRecoveryCoordinationCapacity, PhysicalRecoveryFreshnessPort,
    PhysicalRecoveryRejoinResidentDenial, PhysicalResidencyDimension as Dimension,
    PhysicalRuntimeAdmission, PhysicalStore, QualifiedRecoveryFilesystemMedia,
    RecoveryWalAllocationDenial as Denial,
};
use worth_store_physical_integrity::PhysicalByteRange;

const ORIGINAL: u64 = 16 << 10;

#[test]
fn observation_arc_is_prefunded_and_public_clones_keep_one_real_native_owner() {
    let (_directory, media, coordination) = world();
    let observer = coordination.certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let (layout, _) = Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<ObservationData>())
        .unwrap();
    let arc_bytes = layout.pad_to_align().size() as u64;
    let first_slot_bytes = std::mem::size_of::<PhysicalRecoveryWalIntegrityObservation>() as u64;
    let held = coordination
        .certification_begin_recovery_allocation(
            NonZeroU64::new(ORIGINAL - arc_bytes - first_slot_bytes + 1).unwrap(),
        )
        .unwrap();
    let before = observer.snapshot().for_dimension(dimension);
    let mut denied_builder = WalIntegrityObservationBuilder::new();
    assert_eq!(
        denied_builder.reserve_one(&coordination).unwrap_err(),
        Denial::Backing {
            requested: arc_bytes + first_slot_bytes,
            cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                required: ORIGINAL + 1,
                admitted: ORIGINAL,
            },
        }
    );
    assert_eq!(denied_builder.len(), 0);
    let empty = denied_builder.finish();
    assert!(empty.wal().is_empty());
    assert_eq!(empty.charged_bytes(), 0);
    let denied = observer.snapshot().for_dimension(dimension);
    assert_eq!(denied.admitted_units(), before.admitted_units());
    assert_eq!(denied.admissions(), before.admissions());
    assert_eq!(denied.active_units(), before.active_units());
    assert_eq!(denied.denials(), before.denials() + 1);
    drop(held);

    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    // Inline comparison metadata is sufficient for this storage-only test.
    // No integrity view, candidate or recovery permission is minted here.
    let observation = PhysicalRecoveryWalIntegrityObservation::new(
        PhysicalArtifactScope::current_root_selector(
            media.store_identity(),
            format,
            PhysicalByteRange::new(0, 1).unwrap(),
        ),
        PhysicalRecoveryWalIntegrityObservationOutcome::Admitted,
    );
    let mut builder = WalIntegrityObservationBuilder::new();
    builder.reserve_one(&coordination).unwrap();
    builder.push_reserved(observation);
    builder.reserve_one(&coordination).unwrap();
    builder.push_reserved(observation);
    let sealing_before = observer.snapshot().for_dimension(dimension);
    let owner = builder.finish();
    let ObservationStorage::Shared(data) = &owner.storage else {
        panic!("nonempty observations must share actual backing");
    };
    let vector_bytes = (data.roster.capacity()
        * std::mem::size_of::<PhysicalRecoveryWalIntegrityObservation>())
        as u64;
    let expected = arc_bytes + vector_bytes;
    assert_eq!(owner.charged_bytes(), expected);
    assert_eq!(owner.owned_heap_bytes(), Some(expected));
    assert_eq!(owner.wal(), &[observation, observation]);
    let sealed = observer.snapshot().for_dimension(dimension);
    assert_eq!(sealed.admissions(), sealing_before.admissions());
    assert_eq!(sealed.admitted_units(), sealing_before.admitted_units());
    assert_eq!(sealed.active_units(), expected);
    let clone = owner.clone();
    assert_eq!(clone, owner);
    assert_eq!(clone.wal().as_ptr(), owner.wal().as_ptr());
    let cloned = observer.snapshot().for_dimension(dimension);
    assert_eq!(cloned.admissions(), sealed.admissions());
    assert_eq!(cloned.admitted_units(), sealed.admitted_units());
    assert_eq!(cloned.released_units(), sealed.released_units());
    drop(coordination);
    drop(media);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        expected
    );
    drop(owner);
    assert_eq!(clone.wal(), &[observation, observation]);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        expected
    );
    drop(clone);
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
