use std::num::NonZeroU64;

use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension,
};
use worth_store_physical_format::RecordArtifactFile;

use super::*;
use crate::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecoveryFilesystemMedia, FilesystemAccessPosture, FilesystemMediaAdmission,
    PhysicalRecoveryCoordination, PhysicalRecoveryCoordinationCapacity,
    PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};

mod payload_pressure;

#[test]
fn actual_read_grants_outlive_the_exclusive_window_and_release_with_bytes() {
    let (directory, media, mut coordination) = world();
    write_checkpoint(&directory, &[37; 64]);
    let root_artifact = RecordArtifactFile::RootManifest { generation: 7 };
    let roots = directory.path().join("store/families/records/roots");
    std::fs::create_dir_all(&roots).unwrap();
    std::fs::write(roots.join(root_artifact.file_name()), [19; 48]).unwrap();
    let ports = coordination.residency.ports().clone();
    let mut discovery = media.bounded_discovery(2, 4096).unwrap();
    let mut window = coordination.begin_source_read_allocation().unwrap();
    window.reserve_total(7).unwrap();
    let checkpoint = window.read_checkpoint(&mut discovery, 4096).unwrap();
    let root = window
        .read_checkpoint_source_root(&mut discovery, 7, 4096)
        .unwrap();
    assert_eq!(checkpoint.observed().bytes(), Some(&[37; 64][..]));
    assert_eq!(root.observed().bytes(), Some(&[19; 48][..]));
    assert_eq!(checkpoint.owned_heap_bytes(), Some(64));
    assert_eq!(checkpoint.charged_bytes(), 64);
    assert_eq!(root.owned_heap_bytes(), Some(48));
    assert_eq!(root.charged_bytes(), 48);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        119
    );
    drop(window);
    drop(coordination);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        112
    );
    assert_eq!(checkpoint.observed().bytes().unwrap().len(), 64);
    drop(root);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        64
    );
    drop(checkpoint);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        0
    );
}

#[test]
fn native_path_pressure_denies_before_backend_payload_read_without_refund_admission() {
    let (directory, media, mut coordination) = world();
    write_checkpoint(&directory, &[41; 64]);
    let ports = coordination.residency.ports().clone();
    let limit = coordination.residency.admitted_policy().operation_bytes();
    let held = ports
        .begin_operation(Scope::Maintenance, NonZeroU64::new(limit - 1).unwrap())
        .unwrap();
    let mut discovery = media.bounded_discovery(2, 4096).unwrap();
    let before = ports.allocation_events().snapshot();
    let mut window = coordination.begin_source_read_allocation().unwrap();
    let failure = window.read_checkpoint(&mut discovery, 4096).unwrap_err();
    let RecoveryDiscoveryAllocationFailure::Allocation {
        requested, cause, ..
    } = failure
    else {
        panic!("must deny the actual C4 allocator callback");
    };
    let PhysicalRecoveryObservationAllocationDenial::PathResidency {
        boundary: ArtifactTreePathAllocationBoundary::FileAddress,
        cause: PhysicalRecoveryRejoinResidentDenial::OperationAllocation(failure),
    } = cause
    else {
        panic!("must retain exact native pressure");
    };
    let pressure = failure.pressure().unwrap();
    assert_eq!(
        pressure.dimension(),
        PhysicalResidencyDimension::OperationBytes
    );
    assert!(requested > 1);
    assert_eq!(pressure.requested(), requested as u64);
    assert_eq!(pressure.admitted(), limit - 1);
    assert!(!pressure.effect_may_have_started());
    assert_eq!(discovery.counters().bytes_read, 0);
    assert_eq!(window.charged_bytes(), 0);
    let after = ports.allocation_events().snapshot();
    for dimension in [
        PhysicalResidencyDimension::OperationBytes,
        PhysicalResidencyDimension::OperationScope(Scope::Recovery),
        PhysicalResidencyDimension::TotalBytes,
    ] {
        assert_eq!(
            after.for_dimension(dimension).admissions(),
            before.for_dimension(dimension).admissions()
        );
        assert_eq!(
            after.for_dimension(dimension).admitted_units(),
            before.for_dimension(dimension).admitted_units()
        );
    }
    drop(held);
    let observation = window.read_checkpoint(&mut discovery, 4096).unwrap();
    assert_eq!(observation.observed().bytes(), Some(&[41; 64][..]));
    drop(observation);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        0
    );
}

#[test]
fn absent_and_present_empty_release_transient_paths_but_only_absence_can_escape() {
    let (directory, media, mut coordination) = world();
    let ports = coordination.residency.ports().clone();
    let original = coordination
        .recovery_allocation_admission()
        .unwrap()
        .byte_limit();
    let held = ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(original).unwrap())
        .unwrap();
    let mut discovery = media.bounded_discovery(2, 4096).unwrap();
    let mut window = coordination.begin_source_read_allocation().unwrap();
    assert!(matches!(
        window.read_checkpoint(&mut discovery, 4096),
        Err(RecoveryDiscoveryAllocationFailure::Allocation {
            cause: PhysicalRecoveryObservationAllocationDenial::PathResidency {
                boundary: ArtifactTreePathAllocationBoundary::FileAddress,
                ..
            },
            ..
        })
    ));
    assert_eq!(discovery.counters().bytes_read, 0);
    drop(held);
    let absent = window.read_checkpoint(&mut discovery, 4096).unwrap();
    assert_eq!(absent.charged_bytes(), 0);
    assert_eq!(absent.owned_heap_bytes(), Some(0));
    assert!(absent.into_absent().unwrap().bytes().is_none());
    write_checkpoint(&directory, &[]);
    let empty = window.read_checkpoint(&mut discovery, 4096).unwrap();
    assert_eq!(empty.charged_bytes(), 0);
    assert_eq!(empty.owned_heap_bytes(), Some(0));
    let still_owned = empty.into_absent().unwrap_err();
    assert_eq!(still_owned.observed().bytes(), Some(&[][..]));
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        0
    );
    drop(still_owned);
}

#[test]
fn foreign_discovery_is_rejected_before_callback_or_backend_observation() {
    let (_directory, _media, mut coordination) = world();
    let (_foreign_directory, foreign, _foreign_coordination) = world();
    let ports = coordination.residency.ports().clone();
    let before = ports.allocation_events().snapshot();
    let mut discovery = foreign.bounded_discovery(1, 4096).unwrap();
    let mut window = coordination.begin_source_read_allocation().unwrap();
    let failure = window.read_checkpoint(&mut discovery, 4096).unwrap_err();
    assert!(matches!(
        failure,
        RecoveryDiscoveryAllocationFailure::Allocation {
            requested: 0,
            cause: PhysicalRecoveryObservationAllocationDenial::StoreMismatch,
            ..
        }
    ));
    assert_eq!(discovery.counters(), Default::default());
    assert_eq!(ports.allocation_events().snapshot(), before);
}

fn write_checkpoint(directory: &tempfile::TempDir, bytes: &[u8]) {
    std::fs::write(
        directory.path().join("store/families/checkpoint.current"),
        bytes,
    )
    .unwrap();
}

fn world() -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    world_with_schedule(None)
}

fn world_with_schedule(
    schedule: Option<crate::physical_runtime::certification::MediaFaultSchedule>,
) -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    use worth_proof::TransitionOutcome;
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.clone()).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("actual Store namespace initialization failed"),
    };
    let _ = media.close();
    let qualified = match schedule {
        Some(schedule) => {
            QualifiedRecoveryFilesystemMedia::qualify_existing_for_certification(&root, schedule)
        }
        None => QualifiedRecoveryFilesystemMedia::qualify_existing(&root),
    }
    .unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let capacity = PhysicalRecoveryCoordinationCapacity::admit(2, 4096, 2, 4096)
        .unwrap()
        .with_recovery_allocation_bytes(16 << 10)
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
