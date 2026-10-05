//! Native borrowed Serving reads, without claiming a synthetic C8 custody handoff.

use super::*;
use crate::physical_runtime::ArtifactTreePathAllocationBoundary;

mod selected_inventory;

#[test]
fn borrowed_serving_read_denies_before_address_then_retries_with_same_media_owner() {
    let (directory, recovery_media, coordination) = world();
    let root = directory.path().join("store");
    let wal = root.join("families/wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join(FIRST), [17; PAYLOAD]).unwrap();
    // Relinquish the recovery mutation lease before admitting the actual
    // existing namespace through the ordinary Store media entry surface.
    drop(recovery_media);
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
    let TransitionOutcome::Success(runtime) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("the genuine existing namespace must admit its Serving media owner");
    };
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let metadata = diagnostic_backing::storage_bytes() as u64;
    let held = ports
        .begin_operation(
            Scope::Recovery,
            NonZeroU64::new(ORIGINAL - metadata - 1).unwrap(),
        )
        .unwrap();
    let mut window = PhysicalRecoveryReadAllocation::for_serving(
        &coordination.residency,
        runtime.lifecycle_state().snapshot().generation,
    );
    let failure = window
        .read_serving_wal_payloads(runtime.record_serving_media(), segments(1), 4096)
        .unwrap_err();
    let RecoveryWalReadFailureView::Allocation {
        artifact: RecoveryWalArtifactView::WalDirectory,
        offset,
        requested,
        cause:
            PhysicalRecoveryObservationAllocationDenial::PathResidency {
                boundary: ArtifactTreePathAllocationBoundary::DirectoryAddress,
                cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
            },
    } = failure.diagnostic()
    else {
        panic!("borrowed read must preserve its native pre-address cause: {failure:?}");
    };
    assert_eq!(offset, 0);
    assert!(requested > 1);
    assert_eq!(*required, ORIGINAL - 1 + requested as u64);
    assert_eq!(*admitted, ORIGINAL);
    assert_eq!(failure.charged_bytes(), 0);
    assert_eq!(window.charged_bytes(), 0);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        held.bytes()
    );
    assert_eq!(std::fs::read(wal.join(FIRST)).unwrap(), [17; PAYLOAD]);
    drop(failure);
    drop(held);
    let observed = window
        .read_serving_wal_payloads(runtime.record_serving_media(), segments(1), 4096)
        .unwrap();
    let [artifact] = observed.artifacts() else {
        panic!("exact original file required");
    };
    assert_eq!(artifact.name(), FIRST);
    assert_eq!(artifact.bytes(), Some(&[17; PAYLOAD][..]));
    let retained = size_of::<ObservedWalArtifact>() as u64
        + PAYLOAD as u64
        + artifact.name_heap_bytes() as u64;
    assert_eq!(observed.charged_bytes(), retained);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        retained
    );
    drop(window);
    drop(coordination);
    // Native ownership follows the real result rather than the temporary read
    // window or its original Coordination owner.
    assert_eq!(observed.artifacts()[0].bytes(), Some(&[17; PAYLOAD][..]));
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        retained
    );
    drop(observed);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        0
    );
    runtime.close();
    drop(ports);
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}
