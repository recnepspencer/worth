//! A genuine qualified owner and native pool; payloads exercise read ownership,
//! not the semantic validity of a release-head block or a recovered seal.
use super::*;
use crate::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy, FilesystemAccessPosture,
    FilesystemMediaAdmission, MediaOwnedPhysicalRuntime, PhysicalRecoveryCoordination,
    PhysicalRecoveryCoordinationCapacity, PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission,
    PhysicalStore, QualifiedRecoveryFilesystemMedia,
};
use std::num::NonZeroU64;
use worth_proof::TransitionOutcome;
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};

const ORIGINAL: u64 = 16 << 10;

#[cfg(windows)]
#[test]
fn borrowed_checkpoint_denies_payload_before_read_then_retains_exact_native_backing() {
    let (root, runtime, coordination) = world();
    let media = runtime.record_serving_media();
    let path = root.path().join("store/families/checkpoint.current");
    let input = [53; 8192];
    std::fs::write(&path, input).unwrap();
    let ports = coordination.residency.ports().clone();
    let held = ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(ORIGINAL - 4096).unwrap())
        .unwrap();
    let mut window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let mut observation = media.bounded_record_observation(2, ORIGINAL).unwrap();
    let reads = media
        .counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead);
    let failure = window
        .read_serving_checkpoint(&mut observation, input.len() as u64)
        .unwrap_err();
    assert!(
        matches!(failure,
            RecoveryDiscoveryAllocationFailure::Allocation {
                artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
                offset: 0,
                requested: 8192,
                cause: PhysicalRecoveryObservationAllocationDenial::Residency(
                    PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
                ),
            } if required == ORIGINAL - 4096 + 8192 && admitted == ORIGINAL
        ),
        "checkpoint payload denial must preserve the responsible native cause: {failure:?}"
    );
    assert_eq!(observation.counters().bytes_read, 0);
    assert_eq!(
        media
            .counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead,),
        reads
    );
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        ORIGINAL - 4096
    );
    drop(held);
    let observed = window
        .read_serving_checkpoint(&mut observation, input.len() as u64)
        .unwrap();
    assert_eq!(observed.observed().bytes(), Some(input.as_slice()));
    assert_eq!(observed.charged_bytes(), 8192);
    assert_eq!(observed.owned_heap_bytes(), Some(8192));
    drop(observation);
    drop(window);
    drop(coordination);
    runtime.close();
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        8192
    );
    drop(observed);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        0
    );
    assert_eq!(std::fs::read(path).unwrap(), input);
}

#[cfg(windows)]
#[test]
fn borrowed_head_reread_denies_native_address_backing_then_retries_and_retains_bytes() {
    let (root, runtime, coordination) = world();
    let media = runtime.record_serving_media();
    let ceiling = ArtifactCeiling::page(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        PageAddress::ReleaseCustodyHeadBlock {
            generation: 7,
            block: 2,
        },
    );
    let address = RecordArtifactFile::ReleaseCustodyHeadBlock {
        generation: 7,
        block: 2,
    };
    let directory = root.path().join("store/families/records/roots");
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join(address.file_name());
    let input = [37; 64];
    std::fs::write(&path, input).unwrap();
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    let held = ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(ORIGINAL - 1).unwrap())
        .unwrap();
    let mut window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let mut observation = media.bounded_record_observation(4, 4096).unwrap();
    let before = observer.snapshot();
    let failure = window
        .read_serving_record(&mut observation, ceiling)
        .unwrap_err();
    let RecoveryDiscoveryAllocationFailure::Allocation {
        artifact: RecoveryDiscoveryArtifact::Record(observed),
        offset: 0,
        requested,
        cause:
            PhysicalRecoveryObservationAllocationDenial::PathResidency {
                boundary: ArtifactTreePathAllocationBoundary::DirectoryAddress,
                cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
            },
    } = failure
    else {
        panic!("actual head reread must preserve its first native address denial: {failure:?}");
    };
    assert_eq!(observed, address);
    assert!(requested > 1);
    assert_eq!(required, ORIGINAL - 1 + requested as u64);
    assert_eq!(admitted, ORIGINAL);
    assert_eq!(observation.counters().bytes_read, 0);
    assert_eq!(window.charged_bytes(), 0);
    let recovery = Dimension::OperationScope(Scope::Recovery);
    let after = observer.snapshot();
    assert_eq!(
        after.for_dimension(recovery).admitted_units(),
        before.for_dimension(recovery).admitted_units()
    );
    assert_eq!(
        after.for_dimension(recovery).released_units(),
        before.for_dimension(recovery).released_units()
    );
    assert_eq!(
        after.for_dimension(recovery).denials(),
        before.for_dimension(recovery).denials() + 1
    );
    assert_eq!(after.for_dimension(recovery).active_units(), ORIGINAL - 1);
    assert_eq!(std::fs::read(&path).unwrap(), input);
    drop(held);

    let whole = window
        .read_serving_record(&mut observation, ceiling)
        .unwrap();
    let range = window
        .read_serving_record_range(&mut observation, address, 9, 11)
        .unwrap();
    assert_eq!(whole.observed().bytes(), Some(input.as_slice()));
    assert_eq!(range.observed().bytes(), Some(&input[9..20]));
    assert_eq!(
        whole.charged_bytes(),
        whole.observed().bytes().unwrap().len() as u64
    );
    assert_eq!(range.charged_bytes(), 11);
    assert_eq!(whole.owned_heap_bytes(), Some(64));
    assert_eq!(range.owned_heap_bytes(), Some(11));
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        75
    );
    drop(observation);
    drop(window);
    drop(coordination);
    runtime.close();
    assert_eq!(whole.observed().bytes(), Some(input.as_slice()));
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        75
    );
    drop(range);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        64
    );
    drop(whole);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        0
    );
    assert_eq!(std::fs::read(&path).unwrap(), input);
    drop(ports);
    for dimension in [recovery, Dimension::OperationBytes, Dimension::TotalBytes] {
        let disposed = observer.snapshot().for_dimension(dimension);
        assert_eq!(disposed.active_units(), 0);
        assert_eq!(disposed.admitted_units(), disposed.released_units());
    }
}

fn world() -> (
    tempfile::TempDir,
    MediaOwnedPhysicalRuntime,
    PhysicalRecoveryCoordination,
) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.clone()).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("actual Store initialization must admit");
    };
    media.close();
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(&root).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let admitted = qualified.admit_persisted_store().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &admitted,
            PhysicalRecoveryCoordinationCapacity::admit(2, 4096, 2, 4096)
                .unwrap()
                .with_recovery_allocation_bytes(ORIGINAL)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    // Keep the real registered pool/session; release the recovery media before
    // admitting an ordinary qualified owner of that same persisted namespace.
    drop(admitted);
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("ordinary same-namespace qualification must admit");
    };
    assert_eq!(
        media.store_identity(),
        coordination.residency.ports().store_identity()
    );
    (directory, media, coordination)
}
